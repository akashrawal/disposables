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
package io.p01def.disposables.protocol;

/*
 Ready -> {"kind":"Ready"}
 Exited(Some(0)) -> {"kind":"Exited","data":0}
 Exited(None) -> {"kind":"Exited","data": null}
 FailedToPrepare("Failed to prepare container") -> {"kind":"FailedToPrepare","data":"Failed to prepare container"}
 FailedToStartEntrypoint("Failed to start entrypoint") -> {"kind":"FailedToStartEntrypoint","data":"Failed to start entrypoint"}
 FailedTimeout -> {"kind":"FailedTimeout"
*/

import org.junit.jupiter.api.Test;

import com.fasterxml.jackson.databind.ObjectMapper;

public class V1EventTest {
	@Test
	void deserialize() throws Exception {
		String[] messages = {
			"{\"kind\":\"Ready\"}",
			"{\"kind\":\"Exited\",\"data\":0}",
			"{\"kind\":\"Exited\",\"data\":null}",
			"{\"kind\":\"FailedToPrepare\",\"data\":\"Failed to prepare container\"}",
			"{\"kind\":\"FailedToStartEntrypoint\",\"data\":\"Failed to start entrypoint\"}",
			"{\"kind\":\"FailedTimeout\"}",
		};

		ObjectMapper mapper = new ObjectMapper();

		for (String msg : messages) {
			V1Event event = mapper.readValue(msg, V1Event.class);
			System.out.println(event);
		}
	}
}
