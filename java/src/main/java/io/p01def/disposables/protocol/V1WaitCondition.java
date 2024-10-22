package io.p01def.disposables.protocol;

import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;

import java.util.Arrays;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonSubTypes;

@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, 
	property = "kind")
@JsonSubTypes({
	@JsonSubTypes.Type(value = V1WaitCondition.Port.class, name = "Port"),
	@JsonSubTypes.Type(value = V1WaitCondition.Stdout.class, name = "Stdout"),
	@JsonSubTypes.Type(value = V1WaitCondition.Command.class, name = "Command"),
})
public class V1WaitCondition {
	public static class Port extends V1WaitCondition {
		public int data;

		public Port() {
		}

		public Port(int port) {
			this.data = port;
		}

		@Override
		public String toString() {
			return "Port(" + data + ")";
		}
	}
	
	public static class Stdout extends V1WaitCondition {
		public String data;

		public Stdout() {
		}

		public Stdout(String expr) {
			this.data = expr;
		}

		@Override
		public String toString() {
			return "Stdout(" + data + ")";
		}
	}
	
	public static class Command extends V1WaitCondition {
		public static class Data {
			public String[] argv;
			@JsonProperty("interval_msec")
			public int intervalMsec;
		}
		
		public Data data;

		public Command() {
		}

		public Command(String[] argv, int intervalMsec) {
			this.data = new Data();
			this.data.argv = argv;
			this.data.intervalMsec = intervalMsec;
		}
		
		@Override
		public String toString() {
			return "Command(" + Arrays.toString(data.argv) + ", " + data.intervalMsec + ")";
		}
	}
}
