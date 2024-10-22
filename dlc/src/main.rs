mod ready;

use std::collections::HashSet;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use base64::Engine;
use futures::FutureExt;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;

use disposables_protocol::{V1SetupMsg, V1WaitCondition, V1Event};
use disposables_protocol::V1_ENV_SETUP;
use tokio::sync::mpsc::{Receiver, Sender};

use ready::ReadySignal;

struct MySetupMsg {
    files: Vec<(String, String)>,
    port: u16,
    wait_for: Vec<V1WaitCondition>,
    ready_timeout_s: u64,
    port_check_interval_ms: u64,
    client_timeout_s: u64,
}

impl MySetupMsg {
    fn fetch() -> Self {
        let mut res = Self {
            files: Vec::new(),
            port: 4,
            wait_for: Vec::new(),
            ready_timeout_s: 120,
            port_check_interval_ms: 500,
            client_timeout_s: 15,
        };

        if let Ok(v) = std::env::var(V1_ENV_SETUP) {
            let msg = serde_json::from_str::<V1SetupMsg>(&v)
                .unwrap_or_else(|e| {
                    panic!("Unable to parse {} variable: {e}", V1_ENV_SETUP)
                });
            res.files.extend(msg.files);
            res.port = msg.port;
            res.wait_for = msg.wait_for;
            if let Some(v) = msg.ready_timeout_s {
                res.ready_timeout_s = v;
            }
        }


        res
    }
}

struct Context {
    setup: MySetupMsg,
    arg0: OsString,
    args: Vec<OsString>,
}

async fn read_line(kind: &str, stream: &mut (impl AsyncBufRead + Unpin)) 
-> Option<String> {
    let mut line = String::new();
    let res = stream.read_line(&mut line).await
        .expect("Cannot read from child process");
    if res == 0 { 
        None
    } else {
        let line = line.trim_end().to_owned();
        println!("[{kind}] {line}");
        Some(line)
    }
}

async fn scan_output(ctx: &Context, kind: &str, stream: &mut (impl AsyncBufRead + Unpin),
    ready_signal: &ReadySignal) {
    let mut patterns = Vec::new();
    for condition in &ctx.setup.wait_for {
        if let V1WaitCondition::Stdout(pattern) = condition {
            patterns.push(pattern);
        }
    }

    while !patterns.is_empty() {
        if let Some(line) = read_line(kind, stream).await {
            let rm_list = patterns.iter()
                .filter_map(|p| line.contains(*p).then_some(*p))
                .collect::<HashSet<&String>>();

            let prev_len = patterns.len();
            patterns.retain(|p| !rm_list.contains(p));
            ready_signal.dec((prev_len - patterns.len()) as i32).await;
        } else {
            break;
        }
    }
}

async fn check_ports(ctx: &Context, ready_signal: &ReadySignal) {
    let interval = Duration::from_millis(ctx.setup.port_check_interval_ms);
    let mut futures = Vec::new();

    for condition in &ctx.setup.wait_for {
        if let V1WaitCondition::Port(port) = condition {
            let port = *port;
            let addrs = [(IpAddr::from(Ipv4Addr::LOCALHOST), port),
                (IpAddr::from(Ipv6Addr::LOCALHOST), port)];
            for addr in addrs {
                let fut = async move {
                    loop {
                        let result = TcpStream::connect(addr).await;
                        if result.is_ok() {
                            break;
                        }
                        tokio::time::sleep(interval).await;
                    }
                    ready_signal.dec(1).await;
                };
                futures.push(fut);
            }
        }
    }

    futures::future::join_all(futures).await;
}

async fn check_commands(ctx: &Context, ready_signal: &ReadySignal) {
    let mut futures = Vec::new();

    for condition in &ctx.setup.wait_for {
        if let V1WaitCondition::Command { argv, interval_msec } = condition {
            let mut iter = argv.iter();
            let argv0 = match iter.next() {
                Some(x) => x,
                None => {
                    log::warn!("Empty command given as wait condition");
                    continue;
                }
            };
            let args = iter.collect::<Vec<_>>();
            futures.push(async move {
                loop {
                    let status = Command::new(argv0).args(&args).status().await;
                    match status {
                        Err(e) => {
                            log::warn!("Unable to execute {argv:?}: {e}");
                        },
                        Ok(status) => {
                            if status.success() {
                                ready_signal.dec(1).await; 
                                break;
                            }
                        }
                    }
                    if *interval_msec > 0 {
                        tokio::time::sleep(
                            Duration::from_millis(*interval_msec)).await;
                    } else {
                        break;
                    }
                }
            });
        }
    }

    futures::future::join_all(futures).await;
}

async fn run_entrypoint(ctx: &Context, sender: Sender<V1Event>) {

    let start_res: Result<(), V1Event> = async {
        //Write all files
        for (path, base64) in &ctx.setup.files {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(base64)
                .map_err(|e| format!("Failed to decode {path}: {e}"))
                .map_err(V1Event::FailedToPrepare)?;
            std::fs::write(path, bytes)
                .map_err(|e| format!("Failed to write {path}: {e}"))
                .map_err(V1Event::FailedToPrepare)?;
        }

        //Start the entrypoint
        let mut child = Command::new(&ctx.arg0).args(&ctx.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| V1Event::FailedToStartEntrypoint(e.to_string()))?;

        let stdout = child.stdout.take()
            .expect("stdout of child process is None");
        let mut stdout = BufReader::new(stdout);

        let stderr = child.stderr.take()
            .expect("stderr of child process is None");
        let mut stderr = BufReader::new(stderr);

        let ready_signal = ReadySignal::new(ctx.setup.wait_for.len() as i32, 
            sender.clone());

        futures::select!{
            //Wait till child exits
            _ = async {
                let wait_res = child.wait().await
                    .expect("Failed to wait for child");
                sender.send(V1Event::Exited(wait_res.code())).await
                    .expect("Cannot send event");
            }.fuse() => (),
            _ = async {
                futures::join!{
                    //Check stdout for readiness (and copy)
                    async {
                        scan_output(ctx, "out", &mut stdout, &ready_signal).await;
                        while read_line("out", &mut stdout).await.is_some() { }
                    },
                    //Copy stderr
                    async {
                        while read_line("err", &mut stderr).await.is_some() { }
                    },
                    //Check ports for readiness
                    check_ports(ctx, &ready_signal),
                    //Check commands
                    check_commands(ctx, &ready_signal),
                    //Run the timeout
                    async {
                        let dur = Duration::from_secs(ctx.setup.ready_timeout_s);
                        tokio::time::sleep(dur).await;

                        ready_signal.timeout().await;
                    },
                };
                futures::future::pending::<()>().await;
            }.fuse() => (),
        };

        Ok(())
    }.await;

    if let Err(event) = start_res {
        sender.send(event).await.expect("Cannot send event");
    }
}

async fn handle_client(ctx: &Context, mut receiver: Receiver<V1Event>) {
    //Create TCP listener
    let listen_addr = format!("[::]:{}", ctx.setup.port);
    let listener = TcpListener::bind(&listen_addr).await
        .unwrap_or_else(|e| panic!("Unable to listen on {}: {}",
                listen_addr, e));

    //Accept one connection with timeout.
    let stream = futures::select! {
        res = listener.accept().fuse() => {
            res.expect("Unable to accept connection").0  
        }, 
        _ = tokio::time::sleep(Duration::from_secs(ctx.setup.client_timeout_s))
            .fuse() => {
            panic!("Timeout occured while waiting for connection, stopping");
        }
    };

    let (mut input, mut output) = tokio::io::split(stream);

    futures::select!{
        _ = async {
            while let Some(event) = receiver.recv().await {
                let serialized = serde_json::to_vec(&event)
                    .expect("Cannot serialize event");
                async {
                    output.write_u32(serialized.len() as u32).await?;
                    output.write_all(&serialized).await
                }.await.expect("Cannot send event to client");
            }
            std::future::pending::<()>().await;
        }.fuse() => (),
        _ = async {
            //TODO: Temp code to respond to closing connection
            let _ = input.read_u8().await;
        }.fuse() => (),
    };
}

async fn async_main() {
    //Get the entrypoint
    let mut args = std::env::args_os();
    args.next().expect("Unable to fetch args");
    let cmd = args.next().expect("Command is missing");

    if cmd == "install" {
        let current_exe = std::env::current_exe()
            .expect("Unable to find the current executable");
        let target = PathBuf::from(args.next().expect("Target directory is missing"));

        if let Err(e) = std::fs::create_dir(&target) {
            match e.kind() { ErrorKind::AlreadyExists => {
                //Target directory exists, ignore
            }, _ => {
                panic!("Unable to create target directory: {e}")
            }};
        }
        
        if !std::fs::exists(target.join("dlc"))
            .expect("Unable to check if DLC binary exists") {
            std::fs::copy(current_exe, target.join("dlc"))
                .expect("Unable to install executable");
        }
    } else if cmd == "run" {
        let arg0 = args.next().expect("Entrypoint is missing");
        let args = args.collect::<Vec<_>>();

        let ctx = Context {
            setup: MySetupMsg::fetch(),
            arg0,
            args
        };

        let (sender, receiver) = tokio::sync::mpsc::channel::<V1Event>(1);

        futures::select!{
            _ = async {
                run_entrypoint(&ctx, sender).await;
                std::future::pending::<()>().await;
            }.fuse() => (),
            _ = handle_client(&ctx, receiver).fuse() => ()
        };
    } else {
        panic!("Invalid command {}", cmd.to_string_lossy());
    }
}

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().expect("Unable to build tokio runtime")
        .block_on(async_main());
}
