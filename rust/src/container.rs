/*!
 * Container
 */


use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::process::{Child, ChildStdout, Command, Stdio};

use base64::Engine;
use disposables_protocol::{V1_ENV_SETUP, V1Event, V1SetupMsg, V1WaitCondition};

use crate::args::Args;
use crate::context::{DLC_MOUNT_POINT, ExecError, Context};
use crate::util::try_use;

const DLC_PORT: u16 = 4;

/**
 * A type for storing and manipulating parameters needed to build a container.
 */
pub struct ContainerParams {
    image: String,
    ports: Vec<u16>,
    setup_msg: V1SetupMsg,

    entrypoint: Option<Args>,
    cmd: Option<Args>, 
    env: Vec<(String, String)>,
}


impl ContainerParams {
    /**
     * Creates a new ContainerParams struct for a given image.
     */
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            ports: Vec::new(),
            setup_msg: V1SetupMsg {
                port: DLC_PORT,
                wait_for: Vec::new(),
                ready_timeout_s: None,
                files: Vec::new(),
            },

            entrypoint: None,
            cmd: None,
            env: Vec::new(),
        }
    }

    /**
     * Adds a port to be forwarded from the container to the host.
     */
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.ports.push(port);
        self
    }

    /**
     * Adds a file with a given path and contents to be written at a specific 
     * path.
     */
    pub fn file(&mut self, path: impl Into<String>, bytes: impl AsRef<[u8]>)
    -> &mut Self {
        let base64 = base64::engine::general_purpose::STANDARD
            .encode(bytes.as_ref());
        self.setup_msg.files.push((path.into(), base64));
            
        self
    }

    /**
     * Add a condition to wait for before accepting that the container is ready.
     */
    pub fn wait_for(&mut self, condition: V1WaitCondition) -> &mut Self {
        self.setup_msg.wait_for.push(condition);
        self 
    }

    /**
     * Add a condition to wait for a port to be connectable.
     * When the port is connectable, the container is considered ready.
     *
     * There is no need to also forward the port to the host.
     */
    pub fn wait_for_port(&mut self, port: u16) -> &mut Self {
        self.wait_for(V1WaitCondition::Port(port))
    }

    /**
     * Add a condition to wait for a pattern to be found in the container's 
     * stdout. When the pattern is found, the container is considered ready.
     */
    pub fn wait_for_stdout(&mut self, expr: impl Into<String>) -> &mut Self {
        self.wait_for(V1WaitCondition::Stdout(expr.into()))
    }

    /**
     * Run a command in the container to check if it is ready.
     * When the command returns successfully, the container is considered ready.
     *
     * If `interval_msec` is non-zero then the command is run every
     * `interval_msec` milliseconds. If it is zero then the command is only
     * executed once and then the command is supposed to block till
     * the container is ready.
     */
    pub fn wait_for_cmd(&mut self, args: impl Into<Args>,
        interval_msec: u64) -> &mut Self {
        let args: Args = args.into();
        self.wait_for(V1WaitCondition::Command { 
            argv: args.into_vec(),
            interval_msec
        })
    }

    /**
     * Replaces the container's entrypoint with the given argument list.
     */
    pub fn entrypoint(&mut self, value: Args) -> &mut Self {
        self.entrypoint = Some(value);
        self
    }

    /**
     * Replaces the container's command with the given argument list.
     */
    pub fn cmd(&mut self, value: Args) -> &mut Self {
        self.cmd = Some(value);
        self
    }

    /**
     * Adds an environment variable to the container.
     */
    pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>)
        -> &mut Self {
        self.env.push((key.into(), value.into()));
        self
    }
}

/**
 * A type that represents a running container.
 *
 * When this struct is dropped, the container is also terminated.
 *
 * This struct is also safe to be stored in a global variable, the container
 * is terminated when program exits, crashes, or gets killed.
 */
pub struct Container {
    ctx: Context,
    id: String, 
    port_map: HashMap<u16, String>,
    dlc_conn: TcpStream,
}

///Error while reading from the DLC port.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    /// OS side error while reading from the DLC port.
    #[error("OS side error")]
    System(#[source] std::io::Error),
    /// Error while deserializing the PDU.
    #[error("Error while deserializing PDU")]
    Deserialize(#[source] serde_json::Error),
}

fn read_pdu<T>(stream: &mut impl Read) -> Result<T, ReadError> 
where for<'a> T: serde::Deserialize<'a>
{
    let mut size_buf = [0_u8; 4];
    stream.read_exact(&mut size_buf).map_err(ReadError::System)?;
    let size = u32::from_be_bytes(size_buf);

    let mut pdu_body = vec![0_u8; size as usize];
    stream.read_exact(&mut pdu_body).map_err(ReadError::System)?;

    serde_json::from_slice(&pdu_body).map_err(ReadError::Deserialize)
}

/// Error type for this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Cannot create volume for DLC.
    #[error("Cannot create volume for DLC")]
    CannotCreateVolume(#[source] ExecError),
    /// Cannot pull the container image.
    #[error("Cannot pull the container image")]
    CannotPullImage(#[source] ExecError),
    /// Error while parsing the image metadata.
    #[error("Error while parsing image metadata")]
    CannotParseImageMetadata(#[source] serde_json::Error),
    /// Cannot start the container.
    #[error("Cannot start the container")]
    CannotStartContainer(ExecError),
    /// Cannot find the mapped port.
    #[error("Cannot find the mapped port")]
    CannotFindMappedPort(u16, ExecError),
    /// Cannot parse the mapped port. (`podman port` output)
    #[error("Cannot parse the mapped port")]
    CannotParseMappedPort(String),
    /// Cannot connect to the DLC port.
    #[error("Cannot connect to the DLC port")]
    CannotConnectToDlc(Vec<(String, std::io::Error)>),
    /// Cannot read data from the DLC port.
    #[error("Cannot read data from the DLC port")]
    CannotReadPDU(ReadError),
}

impl ContainerParams {
    /**
     * Creates a new container based on the `ContainerParams` struct
     * using the given context.
     */
    pub fn create_using(&self, ctx: &Context) -> Result<Container, Error> {
        //Find image entrypoint and command
        let image_exists = match ctx.podman(["image", "exists", &self.image]) {
            Ok(_) => true, 
            Err(ExecError::ProgramReturnedUnsuccessfully{..}) => false,
            Err(e) => return Err(Error::CannotPullImage(e)),
        };
        if ! image_exists {
            ctx.podman(["image", "pull", &self.image])
                .map_err(Error::CannotPullImage)?;
        }

        let image_meta_str = ctx.podman(["image", "inspect", &self.image])
            .map_err(Error::CannotPullImage)?;


        #[derive(serde::Deserialize)]
        struct ImageMeta {
            #[serde(rename = "Config")]
            config: ImageMetaConfig,
        }

        //TODO: Tests with images having empty entrypoint and empty command
        #[derive(serde::Deserialize)]
        struct ImageMetaConfig {
            #[serde(rename = "Entrypoint")]
            entrypoint: Vec<String>,
            #[serde(rename = "Cmd")]
            cmd: Vec<String>,
        }

        let [image_meta]: [ImageMeta; 1] = serde_json::from_str(&image_meta_str)
        .map_err(Error::CannotParseImageMetadata)?;

        let img_entrypoint = self.entrypoint.as_ref()
            .map(|x| x.get())
            .unwrap_or(image_meta.config.entrypoint.as_slice());
        let img_cmd = self.cmd.as_ref()
            .map(|x| x.get())
            .unwrap_or(image_meta.config.cmd.as_slice());

        //Setup message
        let setup_msg = serde_json::to_string(&self.setup_msg)
            .expect("Error serializing setup message");

        //Ports
        let ports: Vec<u16>
            = [DLC_PORT].iter().chain(&self.ports).cloned().collect();

        //Start container
        let mut args = Args::from(["run", "-d", "--rm",
            "-v", &format!("{}:{DLC_MOUNT_POINT}", ctx.volume()),
            "-e", &format!("{V1_ENV_SETUP}={setup_msg}")]);
        for (key, value) in &self.env {
            args.add("-e").add(format!("{key}={value}"));
        }
        for p in &ports {
            args.add("-p").add(p.to_string());
        }
        args.add(format!("--entrypoint={}/dlc", ctx.dlc_install_dir()))
            .add(self.image.clone())
            .add("run")
            .extend(img_entrypoint)
            .extend(img_cmd);
        
        let id = match ctx.podman(args.get()) {
            Ok(id) => id,
            Err(_) => {
                ctx.create_volume().map_err(Error::CannotCreateVolume)?;
                ctx.podman(args).map_err(Error::CannotStartContainer)?
            }
        };

        //Create port map
        let mut port_map = HashMap::<u16, String>::new();
        for p in ports {
            let output = ctx.podman(["port", &id, &format!("{p}")])
                .map_err(|e| Error::CannotFindMappedPort(p, e))?;
            port_map.insert(p, output);
        }

        //Connect to DLC port
        let addr_string = port_map.get(&DLC_PORT)
            .expect("DLC port does not exist");
        
        let dlc_conn = try_use(addr_string.split_whitespace(), |x| {
            TcpStream::connect(x).map_err(|e| (x.to_owned(), e))
        }).map_err(Error::CannotConnectToDlc)?;
        
        Ok(Container {
            ctx: ctx.clone(),
            id,
            port_map,
            dlc_conn
        })
    }

    /**
     * Creates a new container based on the `ContainerParams` struct
     * using the global context.
     */
    pub fn create(&self) -> Result<Container, Error> {
        self.create_using(Context::global())
    }
}

impl Container {
    /**
     * Returns the container's ID. The container can be identified
     * by Docker/Podman using the ID.
     *
     * ```rust
     * # use disposables::{ContainerParams, Context};
     * # use disposables::util::try_use;
     * # use disposables::protocol::V1Event;
     *
     * let mut container = ContainerParams::new("docker.io/postgres:16-alpine")
     *     .env("POSTGRES_HOST_AUTH_METHOD", "trust")
     *     .port(5432)
     *     .wait_for_cmd(["pg_isready"], 500)
     *     .create().unwrap();
     *
     * assert!(matches!(container.wait().unwrap(), V1Event::Ready),
     *     "Postgres failed to start: {}", container.logs().unwrap());
     *
     * Context::global().podman(["exec", container.id(),
     *     "createdb", "-U", "postgres", "new_database"]).unwrap();
     * ``` 
     */
    pub fn id(&self) -> &str {
        &self.id
    }

    /**
     * Waits for events from the running container.
     */
    pub fn wait(&mut self) -> Result<V1Event, Error> {
        read_pdu(&mut self.dlc_conn).map_err(Error::CannotReadPDU)
    }

    /**
     * Returns the port mapping for the given port.
     */
    pub fn port(&self, port: u16) -> Option<Vec<&str>> {
        self.port_map.get(&port).map(|x| x.split_whitespace().collect())
    }

    /**
     * Returns the container's logs.
     */
    pub fn logs(&self) -> Result<String, ExecError> {
        self.ctx.podman(["logs", &self.id]) 
    }
    
    /**
     * Opens a stream to the container's logs.
     *
     * The function basically runs `podman logs -f <container_id>`
     * and starts streaming the stdout.
     */
    pub fn logs_stream(&self) -> Result<(ChildStdout, Child), std::io::Error> {
        let mut child = Command::new(self.ctx.engine())
            .args(["logs", "-f", &self.id])
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().expect("stdout is none");
        Ok((stdout, child))
    }
}

