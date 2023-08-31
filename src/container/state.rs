use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ContainerStatus {
  // StateCreating indicates that the container is being created
  Creating,
  // StateCreated indicates that the runtime has finished the create operation
  Created,
  // StateRunning indicates that the container process has executed the
  // user-specified program but has not exited
  Running,
  // StateStopped indicates that the container process has exited
  Stopped,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct State {
  // Version is the version of the specification that is supported.
  pub oci_version: String,
  // ID is the container ID
  pub id: String,
  // Status is the runtime status of the container.
  pub status: ContainerStatus,
  // Pid is the process ID for the container process.
  pub pid: Option<i32>,
  // Bundle is the path to the container's bundle directory.
  pub bundle: String,
  // Annotations are key values associated with the container.
  pub annotations: HashMap<String, String>,
}

impl State {
  pub fn new(
    container_id: &str,
    status: ContainerStatus,
    pid: Option<i32>,
    bundle: &str,
  ) -> Self {
    Self {
      oci_version: "v1.0.0".to_string(),
      id: container_id.to_string(),
      status,
      pid,
      bundle: bundle.to_string(),
      annotations: HashMap::default(),
    }
  }
}