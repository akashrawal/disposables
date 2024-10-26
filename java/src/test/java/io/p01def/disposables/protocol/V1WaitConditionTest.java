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


/*
 
 Port(80) -> {"kind":"Port","data":80}
 Stdout("Hello") -> {"kind":"Stdout","data":"Hello"}
 Command { argv: ["ls", "-l"], interval_msec: 1000 } -> {"kind":"Command","data":{"argv":["ls","-l"],"interval_msec":1000}}
*/

package io.p01def.disposables.protocol;

import org.junit.jupiter.api.Test;

import com.fasterxml.jackson.databind.ObjectMapper;

public class V1WaitConditionTest {
	@Test
	void deserialize() throws Exception {
		String[] messages = {
			"{\"kind\":\"Port\",\"data\":80}",
			"{\"kind\":\"Stdout\",\"data\":\"Hello\"}",
			"{\"kind\":\"Command\",\"data\":{\"argv\":[\"ls\",\"-l\"],\"interval_msec\":1000}}",
		};

		ObjectMapper mapper = new ObjectMapper();

		for (String msg : messages) {
			V1WaitCondition event = mapper.readValue(msg, V1WaitCondition.class);
			System.out.println(event);
		}
	}
}
