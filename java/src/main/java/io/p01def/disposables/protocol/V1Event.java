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
	

