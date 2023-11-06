use std::path::PathBuf;
use std::fs;

use anyhow::{bail, Result};
use clap::Parser;
use nix::sys::signal::Signal;

use crate::container::{Container, ContainerStatus};

#[derive(Debug, Parser)]
pub struct Stop {
    pub container_id: String,
}

impl Stop {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let root_path = fs::canonicalize(root_path)?;
            let container_root = root_path.join(&self.container_id);
            if !container_root.exists() {
                bail!("{} doesn't exists.", self.container_id)
            }
            let mut container = Container::load(container_root)?;
            container.refresh_status()?;
            if container.can_kill() {
                container.do_kill(Signal::SIGKILL)?;
                container.set_status(ContainerStatus::Stopped);
                container.save()?;
                std::process::exit(0)
            } else {
                bail!(
                    "{} counld not be stopped because it was {:?}",
                    container.id(),
                    container.status()
                )
            }
    }
}
