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


