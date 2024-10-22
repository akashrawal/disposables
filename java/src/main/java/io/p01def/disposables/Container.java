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
	public static class MappedPort {
		public final InetAddress addr;
		public final int port;
		public MappedPort(InetAddress addr, int port) {
			this.addr = addr;
			this.port = port;
		}
	}

	public static class ImageMetadataParseException extends Exception {
		public ImageMetadataParseException(String message, Throwable cause) {
			super(message, cause);
		}
	}

	private final Context context;
	private final String id;
	private final HashMap<Integer, ArrayList<MappedPort>> portMap;
	private final Socket dlcConn;	

	@Override
	public void close() throws IOException {
		dlcConn.close();
	}

	public Container(ContainerParams params)
	throws IOException, InterruptedException, ExitStatusException,
			   ImageMetadataParseException {
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
		try {
			Pattern addressAndPortPattern = Pattern.compile("(.+):(\\d+)");
			for (int p : ports) {
				String output = context.podman("port", id, Integer.toString(p));
				String[] parts = output.split("[ \t\n\r]+");
				ArrayList<MappedPort> portList = new ArrayList<>();
				for (String part : parts) {
					if (part.isEmpty()) continue;

					Matcher m = addressAndPortPattern.matcher(part);
					if (!m.matches()) {
						throw new IOException("Cannot parse port mapping: " + part);
					}

					String addrStr = m.group(1);
					int port = Integer.parseInt(m.group(2));
					InetAddress addr = InetAddress.getByName(addrStr);
					portList.add(new MappedPort(addr, port));
				}
				portMap.put(p, portList);
			}
		} catch (Exception e) {
			throw new IOException("Cannot create port map", e);
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

	public V1Event waitForEvent() throws IOException, InterruptedException {
		DataInputStream in = new DataInputStream(dlcConn.getInputStream());		

		int size = in.readInt();
		byte[] data = new byte[size];
		in.readFully(data);

		ObjectMapper mapper = new ObjectMapper();
		return mapper.readValue(data, V1Event.class);
	}

	public String logs() throws IOException, InterruptedException,
		   ExitStatusException {
		return context.podman("logs", "-f", id);
	}

	public List<MappedPort> port(int port) {
		return portMap.get(port);
	}
}


