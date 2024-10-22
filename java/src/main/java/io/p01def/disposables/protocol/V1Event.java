package io.p01def.disposables.protocol;

import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonSubTypes;

@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, property = "kind")
@JsonSubTypes({
	@JsonSubTypes.Type(value = V1Event.Ready.class, name = "Ready"),
	@JsonSubTypes.Type(value = V1Event.Exited.class, name = "Exited"),
	@JsonSubTypes.Type(value = V1Event.FailedToPrepare.class, name = "FailedToPrepare"),
	@JsonSubTypes.Type(value = V1Event.FailedToStartEntrypoint.class, name = "FailedToStartEntrypoint"),
	@JsonSubTypes.Type(value = V1Event.FailedTimeout.class, name = "FailedTimeout"),
})
public class V1Event {
	//enum
	public static class Ready extends V1Event {
		@Override
		public String toString() {
			return "Ready";
		}
	}
	public static class Exited extends V1Event {
		public Integer data;
		@Override
		public String toString() {
			return "Exited(" + data + ")";
		}
	}
	public static class FailedToPrepare extends V1Event {
		public String data;
		@Override
		public String toString() {
			return "FailedToPrepare(" + data + ")";
		}
	}
	public static class FailedToStartEntrypoint extends V1Event {
		public String data;
		@Override
		public String toString() {
			return "FailedToStartEntrypoint(" + data + ")";
		}
	}
	public static class FailedTimeout extends V1Event {
		@Override
		public String toString() {
			return "FailedTimeout";
		}
	}
}
	

