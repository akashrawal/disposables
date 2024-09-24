

use std::io::Read;
use std::net::TcpStream;
use std::process::{Command, Stdio};

use protocol::V1Event;

use super::params::ContainerParams;

/**
 * A type that represents a running container.
 *
 * When this struct is destroyed, the container is also terminated.
 *
 * This struct is also safe to be used as a global variable, the container
 * is terminated when program exits, crashes, or gets killed.
 */

pub struct Container {
    name: String, 
    dlc_conn: TcpStream,
}

#[derive(Debug)]
pub enum ExecError {
    System(std::io::Error),
    Encoding(std::string::FromUtf8Error),
    ProgramReturnedUnsuccessfully(Option<i32>, String),
}

fn run(cmd: &mut Command) -> Result<String, ExecError> {
    let output = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output()
        .map_err(ExecError::System)?;
    if ! output.status.success() {
        return Err(ExecError::ProgramReturnedUnsuccessfully(
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(ExecError::Encoding)
        .map(|s| s.trim().to_owned())
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

pub enum Error {
    CannotStartContainer(ExecError),
    CannotFindDlcPort(ExecError),
    CannotConnectToDlc(String, std::io::Error),
    CannotReadPDU(ReadError),
}

impl Container {
    pub fn new(params: &ContainerParams) -> Result<Self, Error> {
        //TODO: Podman/docker/other implementations portability
        let args = params.start_args();
        let name = run(Command::new("podman").args(args))
            .map_err(Error::CannotStartContainer)?;

        //Connect to DLC port
        let addr_string = run(Command::new("podman")
            .args(["port", &name, "4"]))
            .map_err(Error::CannotFindDlcPort)?;
        
        let dlc_conn = TcpStream::connect(&addr_string)
            .map_err(|e| Error::CannotConnectToDlc(addr_string, e))?;
        
        Ok(Self {
            name,
            dlc_conn
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn wait(&mut self) -> Result<V1Event, Error> {
        read_pdu(&mut self.dlc_conn).map_err(Error::CannotReadPDU)
    }
}
