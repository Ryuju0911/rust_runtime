use std::path::PathBuf;

use crate::container::State;

#[derive(Debug)]
pub struct Container {
  pub state: State,
  pub root: PathBuf
}

impl Container {
  pub fn new() {}
}
