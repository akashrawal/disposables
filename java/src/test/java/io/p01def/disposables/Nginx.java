/*
 * Copyright 2024 Akash Rawal
 *
 * This file is part of Disposables.
 *
 * Disposables is free software: you can redistribute it and/or modify it under 
 * the terms of the GNU General Public License as published by the 
 * Free Software Foundation, either version 3 of the License, or 
 * (at your option) any later version.
 * 
 * Disposables is distributed in the hope that it will be useful, 
 * but WITHOUT ANY WARRANTY; without even the implied warranty of 
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
 * See the GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License 
 * along with Disposables. If not, see <https://www.gnu.org/licenses/>. 
 */

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


