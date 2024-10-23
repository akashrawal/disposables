
package io.p01def.disposables;

import java.net.HttpURLConnection;
import java.net.URL;
import java.util.ArrayList;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import io.p01def.disposables.Container.MappedPort;
import io.p01def.disposables.protocol.V1Event;

public class Nginx {

	@Test
	public void normalServer() throws Exception {
		Container c = new ContainerParams("docker.io/nginx:alpine")
			.port(80)
			.waitForPort(80)
			.file("/usr/share/nginx/html/custom_file.html",
				"<html></html>".getBytes())
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


