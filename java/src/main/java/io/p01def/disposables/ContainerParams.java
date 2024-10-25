package io.p01def.disposables;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;

import io.p01def.disposables.Container.ImageMetadataParseException;
import io.p01def.disposables.Container.SocketAddrParseException;
import io.p01def.disposables.Util.ExitStatusException;
import io.p01def.disposables.protocol.V1SetupMessage;
import io.p01def.disposables.protocol.V1WaitCondition;

/**
 * A type for storing and manipulating parameters needed to build a container.
 */
public class ContainerParams {
	final Context context;
	String image;

	V1SetupMessage setupMsg;
	ArrayList<Integer> ports;
	String[] entrypoint = null;
	String[] cmd = null;
	HashMap<String, String> env = new HashMap<>();

	/**
	 * Creates a new ContainerParams object for a given image.
	 *
	 * @param image The image to use for the container.
	 * @param context The context to use for running the container.
	 */
	public ContainerParams(String image, Context context) {
		this.context = context;
		this.image = image;

		this.setupMsg = new V1SetupMessage();
		this.ports = new ArrayList<>();
	}

	/**
	 * Creates a new ContainerParams object for a given image.
	 *
	 * @param image The image to use for the container.
	 */
	public ContainerParams(String image) {
		this(image, Context.global());
	}

	/**
	 * Adds a port to be forwarded from the container to the host.
	 *
	 * @param port The port to be forwarded.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams port(int port) {
		this.ports.add(port);
		return this;
	}

	/**
	 * Adds a file with a given path and contents to be written at a specific 
	 * path.
	 *
	 * @param path The path to write the file to.
	 * @param bytes The contents of the file.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams file(String path, byte[] bytes) {
		String base64 = Base64.getEncoder().encodeToString(bytes);
		
		this.setupMsg.files.add(new String[]{path, base64});
		return this;
	}

	/**
	 * Add a condition to wait for before accepting that the container is ready.
	 *
	 * @param condition The condition to wait for.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams waitFor(V1WaitCondition condition) {
		this.setupMsg.waitFor.add(condition);
		return this;
	}

	/**
	 * Add a condition to wait for a port to be connectable.
	 * When the port is connectable, the container is considered ready.
	 *
	 * There is no  need to also forward the port to the host.
	 *
	 * @param port The port to wait for.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams waitForPort(int port) {
		return this.waitFor(new V1WaitCondition.Port(port));
	}

	/**
	 * Add a condition to wait for a pattern to be found in the container's 
	 * stdout. When the pattern is found, the container is considered ready.
	 *
	 * @param expr The pattern to wait for.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams waitForStdout(String expr) {
		return this.waitFor(new V1WaitCondition.Stdout(expr));
	}

	/**
	 * Run a command in the container to check if it is ready.
	 * When the command returns successfully, the container is considered ready.
	 *
	 * @param argv The command to run.
	 * @param intervalMsec If greater than 0, then this value represents
	 * 					   the number of milliseconds to wait before 
	 * 					   running the command again (periodic mode).
	 * 					   If this value is zero, the command is only run
	 * 					   once, and the command is supposed to block till
	 * 					   the container is ready.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams waitForCmd(String[] argv, int intervalMsec) {
		return this.waitFor(new V1WaitCondition.Command(argv, intervalMsec));
	}

	/**
	 * Replaces the container's entrypoint with the given argument list.
	 *
	 * @param value The new entrypoint.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams entrypoint(String... value) {
		this.entrypoint = value;
		return this;
	}

	/**
	 * Replaces the container's command with the given argument list.
	 *
	 * @param value The new command.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams cmd(String... value) {
		this.cmd = value;
		return this;
	}

	/**
	 * Adds an environment variable to the container.
	 *
	 * @param key The name of the environment variable.
	 * @param value The value of the environment variable.
	 * @return The ContainerParams object itself, intended for chaining.
	 */
	public ContainerParams env(String key, String value) {
		this.env.put(key, value);
		return this;
	}

	/**
	 * Creates a new container based on the `ContainerParams` object
	 * using the global context.
	 *
	 * @return The created container.
	 * @throws IOException If an I/O error occurs.
	 * @throws ImageMetadataParseException If the image metadata cannot be parsed.
	 * @throws ExitStatusException If the container engine exits with a non-zero 
	 *                             status.
	 * @throws SocketAddrParseException If a socket address from `podman port`
	 *                                  cannot be parsed.
	 * @throws InterruptedException If the thread is interrupted.
	 */
	public Container create() 
			throws IOException, ImageMetadataParseException, ExitStatusException,
			   SocketAddrParseException, InterruptedException {
		return new Container(this);
	}
}
