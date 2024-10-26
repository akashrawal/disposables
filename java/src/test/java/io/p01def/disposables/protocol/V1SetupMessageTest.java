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
{"port":4,"wait_for":[
	{"kind":"Port","data":80},
	{"kind":"Stdout","data":"Hello"},
	{"kind":"Command","data":{"argv":["ls","-l"],"interval_msec":1000}}
],"ready_timeout_s":10,"files":[["file1","base64"]]}
*/

import org.junit.jupiter.api.Test;

import com.fasterxml.jackson.databind.ObjectMapper;

public class V1SetupMessageTest {

	@Test
	public void deserialize() throws Exception {
		String json = 
"{\"port\":4,\"wait_for\":[" + 
"	{\"kind\":\"Port\",\"data\":80}," + 
"	{\"kind\":\"Stdout\",\"data\":\"Hello\"}," + 
"	{\"kind\":\"Command\",\"data\":{\"argv\":[\"ls\",\"-l\"],\"interval_msec\":1000}}" + 
"],\"ready_timeout_s\":10,\"files\":[[\"file1\",\"base64\"]]}";

		ObjectMapper mapper = new ObjectMapper();
		V1SetupMessage msg = mapper.readValue(json, V1SetupMessage.class);
		System.out.println(msg);
		System.out.println("New JSON: " + mapper.writeValueAsString(msg));
	}
}


