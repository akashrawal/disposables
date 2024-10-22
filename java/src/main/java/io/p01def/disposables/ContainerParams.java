package io.p01def.disposables;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;

import io.p01def.disposables.Container.ImageMetadataParseException;
import io.p01def.disposables.Util.ExitStatusException;
import io.p01def.disposables.protocol.V1SetupMessage;
import io.p01def.disposables.protocol.V1WaitCondition;

public class ContainerParams {
	final Context context;
	String image;

	V1SetupMessage setupMsg;
	ArrayList<Integer> ports;
	String[] entrypoint = null;
	String[] cmd = null;
	HashMap<String, String> env = new HashMap<>();

	public ContainerParams(String image, Context context) {
		this.context = context;
		this.image = image;

		this.setupMsg = new V1SetupMessage();
		this.ports = new ArrayList<>();
	}

	public ContainerParams(String image) {
		this(image, Context.global());
	}

	public ContainerParams port(int port) {
		this.ports.add(port);
		return this;
	}

	public ContainerParams file(String path, byte[] bytes) {
		String base64 = Base64.getEncoder().encodeToString(bytes);
		
		this.setupMsg.files.add(new String[]{path, base64});
		return this;
	}

	public ContainerParams waitFor(V1WaitCondition condition) {
		this.setupMsg.waitFor.add(condition);
		return this;
	}

	public ContainerParams waitForPort(int port) {
		return this.waitFor(new V1WaitCondition.Port(port));
	}

	public ContainerParams waitForStdout(String expr) {
		return this.waitFor(new V1WaitCondition.Stdout(expr));
	}

	public ContainerParams waitForCmd(String[] argv, int intervalMsec) {
		return this.waitFor(new V1WaitCondition.Command(argv, intervalMsec));
	}

	public ContainerParams entrypoint(String... value) {
		this.entrypoint = value;
		return this;
	}

	public ContainerParams cmd(String... value) {
		this.cmd = value;
		return this;
	}

	public ContainerParams env(String key, String value) {
		this.env.put(key, value);
		return this;
	}

	public Container create() 
			throws IOException, ImageMetadataParseException, ExitStatusException,
			   InterruptedException {
		return new Container(this);
	}
}
