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

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import java.io.Closeable;
import java.io.DataInputStream;
import java.io.IOException;

import java.net.InetAddress;
import java.net.Socket;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import io.p01def.disposables.Util.ExitStatusException;
import io.p01def.disposables.protocol.V1Event;

/**
 * A type that represents a running container.
 */
public class Container implements Closeable {
	public static class SocketAddrParseException extends Exception {
		public final String socketaddr;
		public SocketAddrParseException(String socketaddr, Throwable cause) {
			super("Cannot parse socket address: " + socketaddr, cause);
			this.socketaddr = socketaddr;
		}
	}

	/**
	 * A type for representing a mapped container port.
	 */
	public static class MappedPort {
		/// The address of the mapped port.
		public final InetAddress addr;

		/// The port number of the mapped port.
		public final int port;

		/**
		 * Creates a new mapped port object.
		 *
		 * @param addr The address of the mapped port.
		 * @param port The port number of the mapped port.
		 */
		public MappedPort(InetAddress addr, int port) {
			this.addr = addr;
			this.port = port;
		}

		/**
		 * Creates a new mapped port object.
		 *
		 * @param socketaddr Mapped port in the format `address:port`.
		 * @throws SocketAddrParseException If the input is invalid.
		 */
		public MappedPort(String socketaddr) throws SocketAddrParseException {
			Pattern addressAndPortPattern = Pattern.compile("(.+):(\\d+)");

			Matcher m = addressAndPortPattern.matcher(socketaddr);
			if (!m.matches()) {
				throw new SocketAddrParseException(socketaddr, null);
			}

			try {
				String addrStr = m.group(1);
				int port = Integer.parseInt(m.group(2));
				InetAddress addr = InetAddress.getByName(addrStr);
				this.addr = addr;
				this.port = port;
			} catch (Exception e) {
				throw new SocketAddrParseException(socketaddr, e);
			}
		}

		/**
		 * Returns the address of the mapped port.
		 *
		 * @return The address of the mapped port.
		 */
		public InetAddress getAddr() {
			return addr;
		}

		/**
		 * String representation of the mapped port.
		 *
		 * @return address:port formatted string.
		 */
		@Override
		public String toString() {
			return addr.getHostAddress() + ":" + port;
		}
	}

	/**
	 * Cannot parse image metadata (as returned by `podman image inspect`).
	 */
	public static class ImageMetadataParseException extends Exception {
		public ImageMetadataParseException(String message, Throwable cause) {
			super(message, cause);
		}
	}

	private final Context context;
	private final String id;
	private final HashMap<Integer, ArrayList<MappedPort>> portMap;
	private final Socket dlcConn;	

	/**
	 * Terminates the container.
	 */
	@Override
	public void close() throws IOException {
		dlcConn.close();
	}

	/**
	 * Creates a new container based on the given parameters.
	 *
	 * @param params The parameters to use for creating the container.
	 * @throws IOException If an I/O error occurs.
	 * @throws InterruptedException If the thread is interrupted.
	 * @throws ExitStatusException If the container engine exits with a non-zero
	 *                             exit code.
	 * @throws ImageMetadataParseException If the image metadata cannot be parsed.
	 * @throws SocketAddrParseException If a socket address from `podman port`
	 *                                  cannot be parsed.
	 */
	public Container(ContainerParams params)
	throws IOException, InterruptedException, ExitStatusException,
			   ImageMetadataParseException, SocketAddrParseException {
		ObjectMapper mapper = new ObjectMapper();

		this.context = params.context;

		//Pull image if it does not exist
		try {
			context.podman("image", "exists", params.image);
		} catch (ExitStatusException e) {
			//Image does not exist
			context.podman("image", "pull", params.image);
		}

		//Inspect image
		String imageMetaStr = context.podman("image", "inspect", params.image);
		ArrayList<String> entrypoint = new ArrayList<String>();
		ArrayList<String> cmd = new ArrayList<String>();
		try {
			JsonNode imageMetaArr = mapper.readTree(imageMetaStr);
			JsonNode imageMeta = imageMetaArr.get(0);
			JsonNode config = imageMeta.get("Config");
			JsonNode entrypointNode = config.get("Entrypoint");
			entrypointNode.elements().forEachRemaining(e -> {
				entrypoint.add(Objects.requireNonNull(e.textValue()));
			});

			JsonNode cmdNode = config.get("Cmd");
			cmdNode.elements().forEachRemaining(e -> {
				cmd.add(Objects.requireNonNull(e.textValue()));
			});
		} catch (Exception e) {
			throw new ImageMetadataParseException("Error parsing image metadata", e);
		}

		//Build the command
		if (params.entrypoint != null) {
			entrypoint.clear();
			Collections.addAll(entrypoint, params.entrypoint);
		}
		if (params.cmd != null) {
			cmd.clear();
			Collections.addAll(cmd, params.cmd);
		}

		String setupMsg = mapper.writeValueAsString(params.setupMsg);

		ArrayList<Integer> ports = new ArrayList<>();
		ports.add(params.setupMsg.port);
		for (int p : params.ports) {
			ports.add(p);
		}

		ArrayList<String> args = new ArrayList<>();
		Collections.addAll(args, "run", "-d", "--rm",
				"-v", context.getVolume() + ":" + Context.DLC_MOUNT_POINT,
				"-e", "DISPOSABLES_V1_SETUP=" + setupMsg);
		for (Map.Entry<String, String> e : params.env.entrySet()) {
			args.add("-e");
			args.add(e.getKey() + "=" + e.getValue());
		}
		for (Integer p : ports) {
			args.add("-p");
			args.add(p.toString());
		}
		args.add("--entrypoint=" + context.dlcInstallDir() + "/dlc");
		args.add(params.image);
		args.add("run");
		args.addAll(entrypoint);
		args.addAll(cmd);
		
		//Start container
		String id;
		try {
			id = context.podman(args.toArray(new String[0]));
		} catch (Exception e) {
			context.createVolume();
			id = context.podman(args.toArray(new String[0]));
		}
		this.id = id;
		
		//Create port map
		portMap = new HashMap<>();
		for (int p : ports) {
			String output = context.podman("port", id, Integer.toString(p));
			String[] parts = output.split("[ \t\n\r]+");
			ArrayList<MappedPort> portList = new ArrayList<>();
			for (String part : parts) {
				if (part.isEmpty()) continue;
				portList.add(new MappedPort(part));
			}
			portMap.put(p, portList);
		}

		//Connect to DLC port
		ArrayList<MappedPort> dlcPortList = portMap.get(params.setupMsg.port);
		Socket newDlcConn = null;
		IOException lastException = null;
		for (MappedPort p : dlcPortList) {
			try {
				newDlcConn = new Socket(p.addr, p.port);
				break;
			} catch (IOException e) {
				lastException = e;
			}
		}
		if (newDlcConn == null) {
			throw new IOException("Cannot connect to DLC port", lastException);
		}
		this.dlcConn = newDlcConn;
	}

	/**
	 * Waits for an event from the container.
	 *
	 * @return The event that was received.
	 * @throws IOException If an I/O error occurs.
	 * @throws InterruptedException If the thread is interrupted.
	 */
	public V1Event waitForEvent() throws IOException, InterruptedException {
		DataInputStream in = new DataInputStream(dlcConn.getInputStream());		

		int size = in.readInt();
		byte[] data = new byte[size];
		in.readFully(data);

		ObjectMapper mapper = new ObjectMapper();
		return mapper.readValue(data, V1Event.class);
	}

	/**
	 * Returns the container's logs.
	 *
	 * @return The container's logs.
	 * @throws IOException If an I/O error occurs.
	 * @throws InterruptedException If the thread is interrupted.
	 * @throws ExitStatusException If `podman logs` exits with a non-zero exit code.
	 */
	public String logs() throws IOException, InterruptedException,
		   ExitStatusException {
		return context.podman("logs", id);
	}

	/**
	 * Returns the port mapping for the given port.
	 *
	 * @param port The port to get the mapping for.
	 * @return The port mapping for the given port.
	 */
	public List<MappedPort> port(int port) {
		return portMap.get(port);
	}
}


