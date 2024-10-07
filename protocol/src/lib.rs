//Protocol related definitions

pub const V1_ENV_SETUP: &str = "DISPOSABLES_V1_SETUP";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum V1WaitCondition {
    Port(u16),
    Stdout(String),
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct V1SetupMsg {
    pub port: u16,
    pub wait_for: Vec<V1WaitCondition>,
    pub ready_timeout_s: Option<u64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum V1Event {
    Ready,
    Exited(Option<i32>),
    FailedToStartEntrypoint(String),
    FailedTimeout,
}
