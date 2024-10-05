
//Library globals: Container engine, DLC container and volume names

use std::process::{Command, Stdio};
use disposables_protocol::DLC_MOUNT_POINT;

use crate::args::Args;

#[derive(Clone)]
pub struct Context {
    engine: String,
    image: String,
    volume: String,
}

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

#[derive(Debug)]
pub enum Error {
    CreateVolume(ExecError),
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
    pub fn podman(&self, args: Args) -> Result<String, ExecError> {
        let output = Command::new(&self.engine).args(args.get())
            .stdout(Stdio::piped()).stderr(Stdio::piped()).output()
            .map_err(ExecError::System)?;
        if ! output.status.success() {
            return Err(ExecError::ProgramReturnedUnsuccessfully{
                args: args.get().iter().map(|s| s.to_owned())
                    .collect(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string()
            });
        }
        String::from_utf8(output.stdout)
            .map_err(ExecError::Encoding)
            .map(|s| s.trim().to_owned())
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

    pub fn new() -> Result<Self, Error>  {
        //TODO: Probe for suitable engine
        //TODO: Allow variables to be changed by environment variables
        let ctx = Self {
            engine: "podman".into(),
            image: "dlc".into(),
            volume: "disposables-dlc".into(),
        };

        //Create volume
        let volume_exists = match ctx.podman(Args::from(["volume", "exists", &ctx.volume])) {
            Ok(_) => true,
            Err(ExecError::ProgramReturnedUnsuccessfully{..}) => false,
            Err(e) => return Err(Error::CreateVolume(e)),
        };
        if ! volume_exists {
            ctx.podman(Args::from(["volume", "create", &ctx.volume]))
                .map_err(Error::CreateVolume)?;
        }
        let install_dir = ctx.dlc_install_dir();
        let volume_spec = format!("{}:/{DLC_MOUNT_POINT}", &ctx.volume);
        ctx.podman(Args::from(["run", "-i", "--rm", "-v", &volume_spec,
            &ctx.image, "install", &install_dir])).map_err(Error::CreateVolume)?;

        Ok(ctx)
    }
}


