
use std::io::{Read, Write};

use disposables::context::Context;
use disposables::container::ContainerParams;
use disposables::protocol::{V1Event, V1WaitCondition};


#[test]
fn normal_server() {
    drop(env_logger::try_init());

    let ctx = Context::new().unwrap();
    let mut container = ContainerParams::new("docker.io/nginx:alpine")
        .port(80)
        .condition(V1WaitCondition::Port(80))
        .create(&ctx).unwrap();

    assert!(matches!(container.wait(), Ok(V1Event::Ready)));

    let mut conn = container.connect_port(80).unwrap();
    write!(conn, "GET / HTTP/1.0\nHost: localhost\n\n").unwrap();

    let mut response_buf = Vec::<u8>::new();
    conn.read_to_end(&mut response_buf).unwrap();
    let response = String::from_utf8(response_buf).unwrap();
    assert_eq!(response.split("\r\n").next().unwrap(), "HTTP/1.1 200 OK",
        "Unexpected response: {response}");
    log::info!("logs: {}", container.logs().unwrap());
}


//TODO: Reading and writing files
//TODO: Delayed startup

