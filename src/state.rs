use std::path::PathBuf;
use std::fs;

use anyhow::Result;
use clap::Parser;

use crate::container::Container;

#[derive(Debug, Parser)]
pub struct State {
    pub container_id: String,
}

impl State {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let root_path = fs::canonicalize(root_path)?;
        let container_root = root_path.join(&self.container_id);
        let mut container = Container::load(container_root)?;
        container.refresh_status()?;
        println!("{}", serde_json::to_string_pretty(&container.state)?);
        std::process::exit(0);
    }
}
