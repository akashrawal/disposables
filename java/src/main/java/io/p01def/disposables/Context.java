
package io.p01def.disposables;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Properties;

import io.p01def.disposables.Util.ExitStatusException;

public class Context {
	private final String engine;
	private final String image;
	private final String volume;

	private static Context globalContext = null;

	static String DLC_MOUNT_POINT = "/dlc";

	public static synchronized Context global() {
		if (globalContext == null) {
			globalContext = new Context();
		}
		return globalContext;
	}

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

	public Context() {
		this(null, null, null);
	}

	public String getEngine() {
		return engine;
	}

	public String getImage() {
		return image;
	}

	public String getVolume() {
		return volume;
	}

	public String podman(String... args)
	throws IOException, InterruptedException, Util.ExitStatusException {
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


