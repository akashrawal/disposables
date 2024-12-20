# Disposables [![Test](https://github.com/akashrawal/disposables/actions/workflows/test.yaml/badge.svg)](https://github.com/akashrawal/disposables/actions/workflows/test.yaml)

Disposables is a library that runs your test dependencies in containers.
Once your tests finish running, the containers are automatically cleaned up.
Disposables works with both Docker and Podman, does not require Docker socket
access, and works well with user namespaces.

Disposables needs Podman or Docker CLI to work.

Disposables is designed with Podman in mind. Unlike Testcontainers,
there is no need to start Podman service, setup environment variables, 
or deal with SELinux, or 'disable ryuk', it just works.

## Rust

Use `disposables` crate.

[![crates.io](https://img.shields.io/crates/v/disposables)](https://crates.io/crates/disposables)
[![Documentation](https://docs.rs/disposables/badge.svg)](https://docs.rs/disposables)

```rust
use disposables::{ContainerParams, Context};
use disposables::util::try_use;
use disposables::protocol::V1Event;
use std::net::TcpStream;
use std::io::{Read, Write};

let mut container = ContainerParams::new("docker.io/nginx:alpine")
    .port(80)  //< Port 80 will be exposed
    .wait_for_port(80) //< Wait for port 80 to be connectable
    .file("/usr/share/nginx/html/custom_file.html",
        "<html>Custom file</html>") //< Add this file before starting entrypoint
    .create().unwrap();

let event = container.wait(); //< Wait for container to become ready
assert!(matches!(event, Ok(V1Event::Ready)),
    "Container start failed: {event:?}, logs: {}", container.logs().unwrap());

//Connect to port 80 of the container
let mut conn = try_use(container.port(80).unwrap(), TcpStream::connect).unwrap();

//Send request
write!(conn, "GET /custom_file.html HTTP/1.0\nHost: localhost\n\n").unwrap();
let mut response_buf = Vec::<u8>::new();
conn.read_to_end(&mut response_buf).unwrap();
let response = String::from_utf8(response_buf).unwrap();
log::info!("Received response {response}"); //< <html>Custom file</html>

//< Container automatically deleted when it goes out of scope.
```

## Java

Use `io.01def:disposables` library. 

[![Maven Central Version](https://img.shields.io/maven-central/v/io.01def/disposables)](https://central.sonatype.com/artifact/io.01def/disposables)
[![Javadoc](https://javadoc.io/badge2/io.01def/disposables/javadoc.svg)](https://javadoc.io/doc/io.01def/disposables)

```java
import java.net.HttpURLConnection;
import java.net.URL;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import io.p01def.disposables.Container.MappedPort;
import io.p01def.disposables.protocol.V1Event;

public class Nginx {

	@Test
	public void normalServer() throws Exception {
		//Use 'docker.io/nginx:alpine' image
		try (Container c = new ContainerParams("docker.io/nginx:alpine")
			.port(80) //< Forward port 80
			.waitForPort(80) //< When port 80 is connectable, the container is ready
			.file("/usr/share/nginx/html/custom_file.html",
				"<html></html>".getBytes()) //< Add this file
			.create();) {

			V1Event event = c.waitForEvent();
			if (! (event instanceof V1Event.Ready)) {
				throw new Exception("Container start failed: " + event 
						+ ", logs: " + c.logs());
			}

			boolean success = false;
			for (MappedPort p : c.port(80)) { //< Try connecting to port 80
				try {
					URL url = new URL("http://" + p + "/custom_file.html");
					System.out.println("Connecting to " + url);
					HttpURLConnection conn = (HttpURLConnection) url.openConnection();
					conn.setRequestMethod("GET");
					int responseCode = conn.getResponseCode();
					if (responseCode != 200) {
						throw new Exception("Unexpected response code: " + responseCode);
					}
					success = true;
					break;
				} catch (Exception e) {
					e.printStackTrace();
				}
			}
			assertTrue(success, "Cannot connect to port");
		} //< Container automatically terminated at this point
	}
}
```
