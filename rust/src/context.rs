/*!
 * Library globals: Container engine, DLC container and volume names
 */

use std::process::{Command, Stdio};

pub(crate) const DLC_MOUNT_POINT: &str = "/dlc";

use crate::args::Args;

/**
 * Errors that can occur while running a command.
 */
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    /// OS side error while running the command.
    #[error("OS side error")]
    System(#[source] std::io::Error),
    /// Error while interpreting the output of the command as UTF-8.
    #[error("Error while interpreting output as UTF-8")]
    Encoding(#[source] std::string::FromUtf8Error),
    /// Program returned unsuccessfully.
    #[error("Program returned unsuccessfully")]
    ProgramReturnedUnsuccessfully{
        /// Arguments passed to the program.
        args: Vec<String>, 
        /// Exit code of the program.
        /// (None if the program was terminated by a signal)
        code: Option<i32>, 
        /// Stderr of the program.
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

/**
 * Stores environment details about which container engine to use
 * and how to use it.
 */
#[derive(Debug, Clone)]
pub struct Context {
    engine: String,
    image: String,
    volume: String,
}

/**
 * Builder pattern implementation for `Context`.
 */
#[derive(Default, Debug)]
pub struct ContextBuilder {
    engine: Option<String>,
    image: Option<String>,
    volume: Option<String>,
}

impl ContextBuilder {
    /**
     * Creates a new builder.
     */
    pub fn new() -> Self {
        Default::default() 
    }

    /**
     * Sets the command for the container engine.
     *
     * The default is fetched from `DISPOSABLES_ENGINE` environment variable.
     * If the environment variable is not set,
     * it will check for `podman` or `docker`, whichever is available.
     * Podman is preferred over Docker.
     */
    pub fn engine(&mut self, value: impl Into<String>) -> &mut Self {
        self.engine = Some(value.into());
        self
    }

    /**
     * Sets the DLC container image to use.
     *
     * The default is fetched from `DISPOSABLES_DLC_IMAGE` environment variable.
     * If the environment variable is not set,
     * `docker.io/akashrawal/dlc:<crate-version>` is used.
     */
    pub fn image(&mut self, value: impl Into<String>) -> &mut Self {
        self.image = Some(value.into());
        self
    }

    /**
     * Sets the volume to be used for storing DLC binary.
     *
     * The default is fetched from `DISPOSABLES_DLC_VOLUME` environment variable.
     * If the environment variable is not set,
     * `docker.io/akashrawal/dlc:<crate-version>` is used.
     */
    pub fn volume(&mut self, value: impl Into<String>) -> &mut Self {
        self.volume = Some(value.into());
        self
    }

    /**
     * Builds the context object.
     */
    pub fn build(&self) -> Result<Context, Error> {
        let maybe_engine = self.engine.clone()
            .or_else(|| std::env::var("DISPOSABLES_ENGINE").ok());
            
        let engine = match maybe_engine {
            Some(engine) => {
                if run(&engine, ["--version"]).is_ok() {
                    engine
                } else {
                    return Err(Error::CannotFindContainerEngine);
                }
            },
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

/// Errors that can occur while creating a context.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Cannot find container engine.
    #[error("Cannot find container engine")]
    CannotFindContainerEngine,
}

impl Context {
    /**
     * Gets the container engine to be used.
     * @see ContextBuilder::engine
     */
    pub fn engine(&self) -> &str {
        &self.engine
    }

    /**
     * Gets the disposables image to be used.
     * @see ContextBuilder::image
     */
    pub fn image(&self) -> &str {
        &self.image
    }

    /**
     * Gets the volume to be used to store the disposables binary.
     * @see ContextBuilder::volume
     */
    pub fn volume(&self) -> &str {
        &self.volume
    }

    /**
     * Executes the container engine with given arguments and captures its
     * output.
     */
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

    /**
     * Gets the default global context.
     */
    pub fn global() -> &'static Self {
        GLOBAL_CONTEXT.get_or_init(|| ContextBuilder::default().build().unwrap())
    }
}


