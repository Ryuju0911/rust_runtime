use std::{path::PathBuf, fs};

use anyhow::{Result, bail};
use clap::Parser;
use nix::sys::signal as nix_signal;

use crate::container::{Container, ContainerStatus};
use crate::signal;

#[derive(Debug, Parser)]
pub struct Kill {
    pub container_id: String,
    pub signal: String,
}

impl Kill {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let root_path = fs::canonicalize(root_path)?;
        let container_root = root_path.join(&self.container_id);
        if !container_root.exists() {
            bail!("{} doesn't exists.", self.container_id)
        }

        let mut container = Container::load(container_root)?;
        container.refresh_status()?;
        if container.can_kill() {
            let sig = signal::from_str(&self.signal.as_str())?;
            nix_signal::kill(container.pid().unwrap(), sig)?;
            container.set_status(ContainerStatus::Stopped).save()?;
            std::process::exit(0)
        } else {
            bail!(
                "{} could not be deleted because it was {:?}",
                container.id(),
                container.status()
            )
        }
    }
}
