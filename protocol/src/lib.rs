//Protocol related definitions

pub const V1_ENV_SETUP: &str = "DISPOSABLES_SETUP_V1";
pub const DEFAULT_LISTEN_ADDR: &str = "[::]:4";

#[derive(serde::Serialize, serde::Deserialize)]
pub enum V1WaitCondition {
    Port(u16),
    Stdout(String),
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct V1SetupMsg {
    pub wait_for: Vec<V1WaitCondition>,
    pub ready_timeout_s: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum V1Event {
    Ready,
    Exited(Option<i32>),
    FailedToStartEntrypoint(String),
    FailedTimeout,
}
