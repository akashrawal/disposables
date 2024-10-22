
#[cfg(test)]
mod nginx;

#[cfg(test)]
mod postgres;

#[cfg(test)]
mod explore {
    use disposables::protocol::*;

    #[test]
    fn explore() {
        drop(env_logger::try_init());

        let messages = [
            V1Event::Ready,
            V1Event::Exited(Some(0)),
            V1Event::Exited(None),
            V1Event::FailedToPrepare("Failed to prepare container".to_owned()),
            V1Event::FailedToStartEntrypoint("Failed to start entrypoint".to_owned()),
            V1Event::FailedTimeout,
        ];
        for msg in messages {
            log::info!("{:?} -> {}", &msg, serde_json::to_string(&msg).unwrap());
        }

        let messages = [
            V1WaitCondition::Port(80),
            V1WaitCondition::Stdout("Hello".to_owned()),
            V1WaitCondition::Command {
                argv: ["ls", "-l"].map(String::from).to_vec(),
                interval_msec: 1000,
            },
        ];
        for msg in messages {
            log::info!("{:?} -> {}", &msg, serde_json::to_string(&msg).unwrap());
        }

        let msg = V1SetupMsg {
            port: 4,
            wait_for: vec![
                V1WaitCondition::Port(80),
                V1WaitCondition::Stdout("Hello".to_owned()),
                V1WaitCondition::Command {
                    argv: ["ls", "-l"].map(String::from).to_vec(),
                    interval_msec: 1000,
                },
            ],
            ready_timeout_s: Some(10),
            files: vec![
                ("file1".to_owned(), "base64".to_owned()),
            ],
        };
        log::info!("{}", serde_json::to_string(&msg).unwrap());
    }
}
