
package io.p01def.disposables;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Properties;

import java.io.IOException;
import java.io.InputStream;

import io.p01def.disposables.Util.ExitStatusException;

/**
 * Stores environment details about which container engine to use
 * and how to use it.
 */
public class Context {
	private final String engine;
	private final String image;
	private final String volume;

	private static Context globalContext = null;

	static String DLC_MOUNT_POINT = "/dlc";

    /**
     * Gets the default global context.
	 *
	 * @return The default global context.
     */
	public static synchronized Context global() {
		if (globalContext == null) {
			globalContext = new Context();
		}
		return globalContext;
	}

    /**
     * Builds a new context object.
	 *
	 * @param engine Path to the container engine to use, or null to auto-detect.
     * @param image The DLC image to use, or null to use the default image.
     * @param volume The DLC volume to use, or null to use the default volume.
     */
	public Context(String engine, String image, String volume) {
		if (engine == null) {
			engine = System.getenv("DISPOSABLES_ENGINE");
		}
		if (engine != null) {
			try {
				Util.run(engine, "--version");
			} catch (Exception e) {
				throw new RuntimeException("Cannot verify container engine " + engine);
			}
		} else {
			try {
				Util.run("podman", "--version");
				engine = "podman";
			} catch (Exception e) {
				try {
					Util.run("docker", "--version");
					engine = "docker";
				} catch (Exception e2) {
					throw new RuntimeException("Cannot find container engine");
				}
			}
		}

		if (image == null) {
			image = System.getenv("DISPOSABLES_DLC_IMAGE");
			if (image == null) {
				String version;
				try (InputStream stream = this.getClass()
						.getResourceAsStream("project-info.properties")) {
					Properties p = new Properties();
					p.load(stream);
					version = p.getProperty("version");
				} catch (Exception _e) {
					//TODO: consider logging
					version = "latest";
				}

				image = "docker.io/akashrawal/disposables-dlc:" + version;
			}
		}

		if (volume == null) {
			volume = System.getenv("DISPOSABLES_DLC_VOLUME");
			if (volume == null) {
				volume = "disposables-dlc";
			}
		}

		this.engine = engine;
		this.image = image;
		this.volume = volume;
	}

	/**
	 * Builds a new context object.
	 *
	 * Equivalent to calling `new Context(null, null, null)`.
	 */
	public Context() {
		this(null, null, null);
	}

	/**
	 * Gets the container engine to be used.
	 *
	 * @return Path to the container engine to use.
	 */
	public String getEngine() {
		return engine;
	}

	/**
	 * Gets the DLC image to be used.
	 *
	 * @return The DLC image to use.
	 */
	public String getImage() {
		return image;
	}

	/**
	 * Gets the DLC volume to be used.
	 *
	 * @return The DLC volume to use.
	 */
	public String getVolume() {
		return volume;
	}

	/**
	 * Executes the container engine with given arguments and captures its
	 * output.
	 *
	 * @param args The arguments to pass to the container engine.
	 * @return The output of the container engine.
	 * @throws IOException If an I/O error occurs while running the container engine.
	 * @throws InterruptedException If the thread is interrupted.
	 * @throws ExitStatusException If the container engine exits with a non-zero
	 *                             exit code.
	 */
	public String podman(String... args)
	throws IOException, InterruptedException, ExitStatusException {
		ArrayList<String> argsList = new ArrayList<>();
		argsList.add(engine);
		argsList.addAll(Arrays.asList(args));
		return Util.run(argsList.toArray(new String[0]));
	}
	
	String dlcInstallDir() {
		StringBuilder sb = new StringBuilder(DLC_MOUNT_POINT + "/");
		for (byte b : this.image.getBytes()) {
			if ((b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z')
					|| (b >= '0' && b <= '9') || b == '_' || b == '-') {
				sb.append(b);
			} else {
				sb.append('_');
			}
		}
		return sb.toString();
	}

	void createVolume() throws IOException, InterruptedException,
		 ExitStatusException {
		try {
			podman("volume", "exists", volume);
		} catch (Util.ExitStatusException e) {
			podman("volume", "create", volume);
		}

		String installDir = dlcInstallDir();
		String volumeSpec = volume + ":" + DLC_MOUNT_POINT;
		podman("run", "-i", "--rm", "-v", volumeSpec,
				this.image, "install", installDir);
	}
}


