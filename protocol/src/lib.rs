/*
 * Copyright 2024 Akash Rawal
 *
 * This file is part of Disposables.
 *
 * Disposables is free software: you can redistribute it and/or modify it under 
 * the terms of the GNU General Public License as published by the 
 * Free Software Foundation, either version 3 of the License, or 
 * (at your option) any later version.
 * 
 * Disposables is distributed in the hope that it will be useful, 
 * but WITHOUT ANY WARRANTY; without even the implied warranty of 
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
 * See the GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License 
 * along with Disposables. If not, see <https://www.gnu.org/licenses/>. 
 */
//!Protocol related definitions for Disposables/DLC

/**
 * Environment variable for setup message.
 */
pub const V1_ENV_SETUP: &str = "DISPOSABLES_V1_SETUP";

/**
 * Enumeration of conditions to wait for before accepting that the container
 * is ready.
 */
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum V1WaitCondition {
    /// Wait for a port to be connectable.
    Port(u16),
    /// Wait for a string to be found in the container's stdout.
    Stdout(String),
    /// Wait for a command to return successfully.
    Command{argv: Vec<String>, interval_msec: u64},
}

/**
 * Description of the setup message for a container.
 *
 * The setup message is serialized in JSON format and passed as an environment
 * variable to the container. (see `V1_ENV_SETUP`)
 */
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct V1SetupMsg {
    /// DLC should listen on this port. Disposables client will connect to 
    /// this port to receive events. When that connection is closed,
    /// DLC will exit.
    pub port: u16,

    /// List of conditions to wait for before accepting that the container
    /// is ready.
    pub wait_for: Vec<V1WaitCondition>,

    /// Timeout for the container to become ready. When the timeout is reached,
    /// the container is considered failed to become ready.
    pub ready_timeout_s: Option<u64>,

    /// List of files to be written before starting the container's entrypoint.
    pub files: Vec<(String, String)>,
}

/**
 * Enumeration of events that can occur in a container.
 *
 * The events are serialized in JSON format and sent to the client.
 */
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum V1Event {
    /// The container is ready to use.
    Ready,
    /// The container's entrypoint has exited.
    Exited(Option<i32>),
    /// Failed to prepare the container.
    FailedToPrepare(String),
    /// Failed to start the container's entrypoint.
    FailedToStartEntrypoint(String),
    /// Timeout occured while waiting for the container to become ready.
    FailedTimeout,
}


