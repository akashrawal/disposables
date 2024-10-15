
//Library globals: Container engine, DLC container and volume names

use std::process::{Command, Stdio};

pub const DLC_MOUNT_POINT: &str = "/dlc";

use crate::args::Args;

#[derive(Debug)]
pub enum ExecError {
    System(std::io::Error),
    Encoding(std::string::FromUtf8Error),
    ProgramReturnedUnsuccessfully{
        args: Vec<String>, 
        code: Option<i32>, 
        stderr: String
    },
}

fn run(arg0: impl Into<String>, args: impl Into<Args>) -> Result<String, ExecError> {
    let arg0 = arg0.into();
    let args = args.into();
    let output = Command::new(&arg0).args(args.get())
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output()
        .map_err(ExecError::System)?;
    if ! output.status.success() {
        return Err(ExecError::ProgramReturnedUnsuccessfully{
            args: [&arg0].into_iter().chain(args.get().iter())
                .map(String::to_owned)
                .collect(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string()
        });
    }
    String::from_utf8(output.stdout)
        .map_err(ExecError::Encoding)
        .map(|s| s.trim().to_owned())
}

#[derive(Clone)]
pub struct Context {
    engine: String,
    image: String,
    volume: String,
}

#[derive(Default)]
pub struct ContextBuilder {
    engine: Option<String>,
    image: Option<String>,
    volume: Option<String>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Default::default() 
    }

    pub fn engine(&mut self, value: impl Into<String>) -> &mut Self {
        self.engine = Some(value.into());
        self
    }

    pub fn image(&mut self, value: impl Into<String>) -> &mut Self {
        self.image = Some(value.into());
        self
    }

    pub fn volume(&mut self, value: impl Into<String>) -> &mut Self {
        self.volume = Some(value.into());
        self
    }

    pub fn build(&self) -> Result<Context, Error> {
        let maybe_engine = self.engine.clone()
            .or_else(|| std::env::var("DISPOSABLES_ENGINE").ok());
            
        let engine = match maybe_engine {
            Some(engine) => engine, 
            None => {
                if run("podman", ["--version"]).is_ok() {
                    "podman"
                } else if run("docker", ["--version"]).is_ok() {
                    "docker"
                } else {
                    return Err(Error::CannotFindContainerEngine);
                }.into()
            }
        };

        Ok(Context {
            engine,
            image: self.image.clone()
                .or_else(|| std::env::var("DISPOSABLES_DLC_IMAGE").ok())
                .unwrap_or(concat!("docker.io/akashrawal/disposables-dlc:",
                        std::env!("CARGO_PKG_VERSION")).into()),
            volume: self.volume.clone()
                .or_else(|| std::env::var("DISPOSABLES_DLC_VOLUME").ok())
                .unwrap_or("disposables-dlc".into()),
        })
    }
}

static GLOBAL_CONTEXT: std::sync::OnceLock<Context> 
    = std::sync::OnceLock::new();

#[derive(Debug)]
pub enum Error {
    CannotFindContainerEngine,
}

impl Context {
    pub fn engine(&self) -> &str {
        &self.engine
    }
    pub fn image(&self) -> &str {
        &self.image
    }
    pub fn volume(&self) -> &str {
        &self.volume
    }
    pub fn podman(&self, args: impl Into<Args>) -> Result<String, ExecError> {
        run(&self.engine, args)
    }

    pub(crate) fn dlc_install_dir(&self) -> String {
        let mut res = format!("{DLC_MOUNT_POINT}/");
        for c in self.image.chars() {
            res.push(match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => c,
                _ => '_'
            });
        }
        res
    }

    pub(crate) fn create_volume(&self) -> Result<(), ExecError> {
        let volume_exists = match self.podman(["volume", "exists", self.volume()]) {
            Ok(_) => true,
            Err(ExecError::ProgramReturnedUnsuccessfully{..}) => false,
            Err(e) => return Err(e),
        };
        if ! volume_exists {
            self.podman(["volume", "create", self.volume()])?;
        }
        let install_dir = self.dlc_install_dir();
        let volume_spec = format!("{}:{DLC_MOUNT_POINT}", self.volume());
        self.podman(["run", "-i", "--rm", "-v", &volume_spec,
            self.image(), "install", &install_dir])?;

        Ok(())
    }

    pub fn new() -> Result<Self, Error>  {
        ContextBuilder::default().build()
    }

    pub fn global() -> &'static Self {
        GLOBAL_CONTEXT.get_or_init(|| Context::new().unwrap())
    }
}


