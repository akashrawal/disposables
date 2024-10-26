
package io.p01def.disposables;

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
		Container c = new ContainerParams("docker.io/nginx:alpine")
			.port(80) //< Forward port 80
			.waitForPort(80) //< When port 80 is connectable, the container is ready
			.file("/usr/share/nginx/html/custom_file.html",
				"<html></html>".getBytes()) //< Add this file
			.create();

		V1Event event = c.waitForEvent();
		if (! (event instanceof V1Event.Ready)) {
			throw new Exception("Container start failed: " + event 
					+ ", logs: " + c.logs());
		}

		boolean success = false;
		for (MappedPort p : c.port(80)) {
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
	}
}


