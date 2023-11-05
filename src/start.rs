use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;
use nix::unistd;

use crate::{container::{Container, ContainerStatus}, notify_socket::NotifySocket};

#[derive(Debug, Parser)]
pub struct Start {
    pub container_id: String,
}

impl Start {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        log::debug!("{} is starting.", self.container_id);
        let container_root = root_path.join(&self.container_id);
        if !container_root.exists() {
            bail!("{} doesn't exists.", self.container_id)
        }
        let mut container = Container::load(container_root)?;
        container.refresh_status()?;
        if !container.can_start() {
            let err_msg = format!(
                "{} could not be started because it was {:?}",
                container.id(),
                container.status()
            );
            log::error!("{}", err_msg);
            bail!(err_msg);
        }

        unistd::chdir(container.root.as_os_str())?;

        let mut notify_socket = NotifySocket::new(&container.root)?;
        notify_socket.notify_container_start()?;

        container.set_status(ContainerStatus::Running);
        container.save()?;

        log::debug!("{} started.", self.container_id);
        Ok(())
    }
}
