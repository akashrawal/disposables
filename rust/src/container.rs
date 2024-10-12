

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

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.ports.push(port);
        self
    }

    pub fn file(&mut self, path: impl Into<String>, bytes: impl AsRef<[u8]>)
    -> &mut Self {
        let base64 = base64::engine::general_purpose::STANDARD
            .encode(bytes.as_ref());
        self.setup_msg.files.push((path.into(), base64));
            
        self
    }

    pub fn wait_for(&mut self, condition: V1WaitCondition) -> &mut Self {
        self.setup_msg.wait_for.push(condition);
        self 
    }

    pub fn wait_for_port(&mut self, port: u16) -> &mut Self {
        self.wait_for(V1WaitCondition::Port(port))
    }

    pub fn wait_for_stdout(&mut self, expr: impl Into<String>) -> &mut Self {
        self.wait_for(V1WaitCondition::Stdout(expr.into()))
    }

    pub fn wait_for_cmd(&mut self, args: impl Into<Args>,
        interval_msec: u64) -> &mut Self {
        let args: Args = args.into();
        self.wait_for(V1WaitCondition::Command { 
            argv: args.into_vec(),
            interval_msec
        })
    }

    pub fn entrypoint(&mut self, value: Args) -> &mut Self {
        self.entrypoint = Some(value);
        self
    }

    pub fn cmd(&mut self, value: Args) -> &mut Self {
        self.cmd = Some(value);
        self
    }

    pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>)
        -> &mut Self {
        self.env.push((key.into(), value.into()));
        self
    }
}

/**
 * A type that represents a running container.
 *
 * When this struct is destroyed, the container is also terminated.
 *
 * This struct is also safe to be used as a global variable, the container
 * is terminated when program exits, crashes, or gets killed.
 */

pub struct Container {
    ctx: Context,
    id: String, 
    port_map: HashMap<u16, String>,
    dlc_conn: TcpStream,
}

#[derive(Debug)]
pub enum ReadError {
    System(std::io::Error),
    Deserialize(serde_json::Error),
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

#[derive(Debug)]
pub enum Error {
    CannotCreateVolume(ExecError),
    CannotPullImage(ExecError),
    CannotParseImageMetadata(serde_json::Error),
    CannotStartContainer(ExecError),
    CannotFindMappedPort(u16, ExecError),
    CannotParseMappedPort(String),
    CannotConnectToDlc(Vec<(String, std::io::Error)>),
    CannotReadPDU(ReadError),
}

impl ContainerParams {
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

        let dlc_path = format!("{}/dlc", ctx.dlc_install_dir());

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
        args.add(format!("--entrypoint={dlc_path}"))
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

    pub fn create(&self) -> Result<Container, Error> {
        self.create_using(Context::global())
    }
}

impl Container {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn wait(&mut self) -> Result<V1Event, Error> {
        read_pdu(&mut self.dlc_conn).map_err(Error::CannotReadPDU)
    }

    pub fn port(&self, port: u16) -> Option<Vec<&str>> {
        self.port_map.get(&port).map(|x| x.split_whitespace().collect())
    }

    pub fn logs(&self) -> Result<String, ExecError> {
        self.ctx.podman(["logs", &self.id]) 
    }
    
    pub fn logs_stream(&self) -> Result<(ChildStdout, Child), std::io::Error> {
        let mut child = Command::new(self.ctx.engine())
            .args(["logs", "-f", &self.id])
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().expect("stdout is none");
        Ok((stdout, child))
    }
}

