use std::{fs, path::PathBuf};

use anyhow::Result;
use nix::unistd::Pid;
use nix::sys::signal::{self, Signal};
use procfs::process::Process;

use crate::container::{ContainerStatus, State};

#[derive(Debug)]
pub struct Container {
    pub state: State,
    pub root: PathBuf,
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

    pub fn set_status(&mut self, status: ContainerStatus) -> &mut Self {
        self.state.status = status;
        self
    }

    pub fn refresh_status(&mut self) -> Result<()> {
        let new_status = match self.pid() {
            Some(pid) => {
                if let Ok(proc) = Process::new(pid.as_raw()) {
                    use procfs::process::ProcState;
                    match proc.stat.state().unwrap() {
                        ProcState::Zombie | ProcState::Dead => ContainerStatus::Stopped,
                        _ => match self.status() {
                            ContainerStatus::Creating | ContainerStatus::Created => self.status(),
                            _ => ContainerStatus::Running,
                        },
                    }
                } else {
                    ContainerStatus::Stopped
                }
            }
            None => ContainerStatus::Stopped,
        };

        self.set_status(new_status);
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        self.state.save(&self.root)
    }

    pub fn can_delete(&self) -> bool {
        self.state.status.can_delete()
    }

    pub fn pid(&self) -> Option<Pid> {
        self.state.pid.map(Pid::from_raw)
    }

    pub fn set_pid(&self, pid: i32) -> Self {
        Self::new(
            self.state.id.as_str(),
            self.state.status,
            Some(pid),
            &self.state.bundle.as_str(),
            &self.root,
        )
        .expect("unexpected error")
    }

    pub fn load(container_root: PathBuf) -> Result<Self> {
        let state = State::load(&container_root)?;
        Ok(Self {
            state,
            root: container_root,
        })
    }

    pub fn do_kill(&mut self, sig: Signal) -> Result<()> {
        signal::kill(self.pid().unwrap(), sig)?;
        Ok(())
    }
}
