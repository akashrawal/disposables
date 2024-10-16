#![warn(missing_docs)]

/*!
 * Disposables is a library that runs your test dependencies in containers.
 * Once your tests finish running, the containers are automatically cleaned up.
 * Disposables works with both Docker and Podman, does not require Docker socket 
 * access, and works well with user namespaces.
 * 
 * Disposables needs Podman or Docker CLI to work.
 * 
 * Generally, this is how the flow goes:
 * 1. Create a container.
 * 2. Wait for the container to become ready. This can be done by checking if
 *    a port is connectable, or certain pattern is found within the container's
 *    stdout.
 * 3. Use the container.
 * 4. Drop the container. Once the container struct is dropped, the container
 *    is terminated.
 * 
 * Sharing one container across multiple tests is possible if you set the struct 
 * as a static variable, the container is cleaned up after all tests have
 * finished.
 *
 * ```rust
 * # use disposables::{ContainerParams, Context};
 * # use disposables::util::try_use;
 * # use disposables::protocol::V1Event;
 * # use std::net::TcpStream;
 * # use std::io::{Read, Write};
 *
 * let mut container = ContainerParams::new("docker.io/nginx:alpine")
 *     .port(80)  //< Port 80 will be exposed
 *     .wait_for_port(80) //< Wait for port 80 to be connectable
 *     .file("/usr/share/nginx/html/custom_file.html",
 *         "<html>Custom file</html>") //< Add this file before starting entrypoint
 *     .create().unwrap();
 * 
 * let event = container.wait(); //< Wait for container to become ready
 * assert!(matches!(event, Ok(V1Event::Ready)),
 *     "Container start failed: {event:?}, logs: {}", container.logs().unwrap());
 * 
 * //Connect to port 80 of the container
 * let mut conn = try_use(container.port(80).unwrap(), TcpStream::connect).unwrap();
 * 
 * //Send request
 * write!(conn, "GET /custom_file.html HTTP/1.0\nHost: localhost\n\n").unwrap();
 * let mut response_buf = Vec::<u8>::new();
 * conn.read_to_end(&mut response_buf).unwrap();
 * let response = String::from_utf8(response_buf).unwrap();
 * log::info!("Received response {response}"); //< <html>Custom file</html>
 * ```
 */

pub mod util;
#[cfg(feature = "async")]
pub mod async_util;

pub mod args;
pub mod context;
pub mod container;

/// Disposables protocol re-exports
pub mod protocol {
    pub use disposables_protocol::*;
}
pub use context::Context;
pub use container::{Container, ContainerParams};
