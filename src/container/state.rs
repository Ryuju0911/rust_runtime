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
  pub oci_version: String,
  pub id: String,
  pub status: ContainerStatus,
  pub pid: Option<i32>,
  pub bundle: String,
  pub annotations: HashMap<String, String>,
}
