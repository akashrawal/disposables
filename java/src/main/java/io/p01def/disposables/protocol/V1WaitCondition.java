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
