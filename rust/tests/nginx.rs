
use std::io::{Read, Write};
use std::net::TcpStream;

use disposables::container::ContainerParams;
use disposables::protocol::V1Event;
use disposables::util::try_use;


#[test]
fn normal_server() {
    drop(env_logger::try_init());

    let mut container = ContainerParams::new("docker.io/nginx:alpine")
        .port(80)
        .wait_for_port(80)
        .create().unwrap();

    assert!(matches!(container.wait(), Ok(V1Event::Ready)),
        "Container start failed, logs: {}", container.logs().unwrap());

    let mut conn = try_use(container.port(80).unwrap(), TcpStream::connect).unwrap();
    write!(conn, "GET / HTTP/1.0\nHost: localhost\n\n").unwrap();

    let mut response_buf = Vec::<u8>::new();
    conn.read_to_end(&mut response_buf).unwrap();
    let response = String::from_utf8(response_buf).unwrap();
    assert_eq!(response.split("\r\n").next().unwrap(), "HTTP/1.1 200 OK",
        "Unexpected response: {response}");
}

#[test]
fn custom_file() {
    drop(env_logger::try_init());

    let mut container = ContainerParams::new("docker.io/nginx:alpine")
        .port(80)
        .wait_for_port(80)
        .file("/usr/share/nginx/html/test.html",
            "<html></html>")
        .create().unwrap();

    let event = container.wait();
    assert!(matches!(event, Ok(V1Event::Ready)),
        "Container start failed: {event:?}, logs: {}", container.logs().unwrap());

    let mut conn = try_use(container.port(80).unwrap(), TcpStream::connect).unwrap();
    write!(conn, "GET /test.html HTTP/1.0\nHost: localhost\n\n").unwrap();

    let mut response_buf = Vec::<u8>::new();
    conn.read_to_end(&mut response_buf).unwrap();
    let response = String::from_utf8(response_buf).unwrap();
    assert_eq!(response.split("\r\n").next().unwrap(), "HTTP/1.1 200 OK",
        "Unexpected response: {response}");
}

//TODO: Delayed startup

