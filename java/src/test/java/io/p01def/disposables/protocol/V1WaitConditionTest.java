

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
