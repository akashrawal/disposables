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

package io.p01def.disposables;

import java.io.IOException;
import java.io.InputStream;

public class Util {
	private static class StreamCollector implements Runnable {
		private InputStream in;
		private StringBuilder res;
		private IOException e;
		public StreamCollector(InputStream in) {
			this.in = in;
			this.res = new StringBuilder();
			this.e = null;
		}
		@Override
		public void run() {
			try {
				byte[] buf = new byte[1024];
				int len = 0;
				while ((len = in.read(buf)) > 0) {
					res.append(new String(buf, 0, len));
				}
			} catch (IOException e) {
				this.e = e;
			}
		}

		public String getResult() throws IOException {
			if (e != null) {
				throw e;
			}
			return res.toString();
		}
	}

	public static class ExitStatusException extends Exception {
		private final int code;

		public ExitStatusException(String[] args, int code, String stderr) {
			super("Program [" + String.join(" ", args)
					+ "] exited with code " + code + ": " + stderr);
			this.code = code;
		}

		public int getCode() {
			return code;
		}
	}
	
	public static String run(String... args) 
			throws IOException, InterruptedException, ExitStatusException {
		ProcessBuilder b = new ProcessBuilder(args);
		b.redirectOutput(ProcessBuilder.Redirect.PIPE);
		b.redirectError(ProcessBuilder.Redirect.PIPE);

		Process p = b.start();
		StreamCollector out = new StreamCollector(p.getInputStream());
		Thread outThread = new Thread(out);
		outThread.start();
		StreamCollector err = new StreamCollector(p.getErrorStream());
		Thread errThread = new Thread(err);
		errThread.start();

		outThread.join();
		errThread.join();

		int status = p.waitFor();
		if (status != 0) {
			throw new ExitStatusException(args, status, err.getResult().trim());
		}
		return out.getResult().trim();
	}
}


