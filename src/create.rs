use std::{path::PathBuf, fs};

use anyhow::{bail, Result};
use clap::Parser;
use nix::unistd;

use crate::container::{Container, ContainerStatus};

#[derive(Parser, Debug)]
pub struct Create {
  #[clap(short, long)]
  pid_file: Option<String>,
  #[clap(short, long, default_value=".")]
  bundle: PathBuf,
  #[clap(short, long)]
  console_socket: Option<String>,
  pub container_id: String,
}

impl Create {
  pub fn exec(&self, root_path: PathBuf) -> Result<()> {
    let container_dir = root_path.join(&self.container_id);
    if !container_dir.exists() {
      fs::create_dir(&container_dir).unwrap();
    } else {
      bail!("{} already exists", self.container_id);
    }

    unistd::chdir(&self.bundle)?;

    let container_dir = fs::canonicalize(container_dir)?;
    unistd::chdir(&*container_dir)?;

    let container = Container::new(
      &self.container_id,
      ContainerStatus::Creating,
      None,
      self.bundle.to_str().unwrap(),
      &container_dir,
    )?;
    container.save()?;
    Ok(())
  }
}