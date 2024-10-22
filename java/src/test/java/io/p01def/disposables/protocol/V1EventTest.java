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
