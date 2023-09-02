use std::{path::PathBuf, fs};

use anyhow::Result;

use crate::container::{ContainerStatus, State};


#[derive(Debug)]
pub struct Container {
  pub state: State,
  pub root: PathBuf
}

impl Container {
  pub fn new(
    container_id: &str,
    status: ContainerStatus,
    pid: Option<i32>,
    bundle: &str,
    container_root: &PathBuf,
  ) -> Result<Self> {
    let container_root = fs::canonicalize(container_root)?;
    let state = State::new(container_id, status, pid, bundle);
    Ok(Self {
      state,
      root: container_root,
    })
  }

  pub fn id(&self) -> &str {
    self.state.id.as_str()
  }

  pub fn status(&self) -> ContainerStatus {
    self.state.status
  }
}
