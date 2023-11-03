use std::{path::PathBuf, fs};

use anyhow::{Result, bail};
use clap::Parser;
use nix::sys::signal;

use crate::container::{Container, ContainerStatus};

/// Release any resources held by the container
#[derive(Debug, Parser)]
pub struct Delete {
    pub container_id: String,
    /// forces deletion of the container if it is still running (using SIGKILL)
    #[clap(short, long)]
    pub force: bool,
}

impl Delete {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let root_path = fs::canonicalize(root_path)?;
        let container_root = root_path.join(&self.container_id);
        if !container_root.exists() {
            bail!("{} doesn't exists.", self.container_id)
        }
        
        let mut container = Container::load(container_root)?;
        container.refresh_status()?;
        
        // Check if container is allowed to be deleted based on container status.
        match container.status() {
            ContainerStatus::Stopped => {}
            _ => {
                // Containers can't be deleted while in these status, unless
                // force flag is set. In the force case, we need to clean up any
                // processes associated with containers.
                if self.force {
                    container.do_kill(signal::Signal::SIGKILL)?;
                    container.set_status(ContainerStatus::Stopped).save()?;
                } 
            }
        }
        if container.can_delete() {
            if container.root.exists() {
                fs::remove_dir_all(&container.root)?;
            }
            log::debug!("{} was deleted successfully", container.id());
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
