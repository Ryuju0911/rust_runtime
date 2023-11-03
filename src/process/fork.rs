use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use anyhow::Result;
use anyhow::bail;
use child::ChildProcess;
use init::InitProcess;
use nix::sched;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd;

use crate::cond::Cond;
use crate::container::Container;
use crate::container::ContainerStatus;
use crate::process::{child, init, parent, Process};
use crate::spec;

pub fn fork_first<P: AsRef<Path>>(
    pid_file: Option<P>,
    userns: bool,
    linux: &spec::Linux,
    container: &mut Container,
) -> Result<Process> {
    let ccond = Cond::new()?;

    let (mut parent, sender_for_parent) = parent::ParentProcess::new()?;
    let child = child::ChildProcess::new(sender_for_parent)?;
    
    unsafe {
        match unistd::fork()? {
            unistd::ForkResult::Child => {
                if let Some(ref r) = linux.resources {
                    if let Some(adj) = r.oom_score_adj {
                        let mut f = fs::File::create("/proc/self/oom_score_adj")?;
                        f.write_all(adj.to_string().as_bytes())?;
                    }
                }

                /*
                 * Deal with user namespaces first. They are quite special, as they
                 * affect our ability to unshare other namespaces and are used as
                 * context for privilege checks.
                 *
                 * Also, there are couple of inconsistency behaviour in vairous kernels,
                 * unsharing alll namespaces together results into incorrect namespace object.
                 */
                if userns {
                    sched::unshare(sched::CloneFlags::CLONE_NEWUSER)?;
                }

                ccond.notify()?;

                Ok(Process::Child(child))
            }
            unistd::ForkResult::Parent { child } => {
                ccond.wait()?;

                let init_pid = parent.wait_for_child_ready()?;
                container
                    .set_status(ContainerStatus::Created)
                    .set_pid(init_pid)
                    .save()?;
                if let Some(pid_file) = pid_file {
                    fs::write(&pid_file, format!("{}", child))?;
                }
                Ok(Process::Parent(parent))
            }
        }
    }
}

pub fn fork_init(mut child_process: ChildProcess) -> Result<Process>  {
    let sender_for_child = child_process.setup_uds()?;
    unsafe {
        match unistd::fork()? {
            unistd::ForkResult::Child => Ok(Process::Init(InitProcess::new(sender_for_child))),
            unistd::ForkResult::Parent { child } => {
                child_process.wait_for_init_ready()?;
                child_process.ready(child)?;

                match waitpid(child, None)? {
                    WaitStatus::Exited(pid, status) => {
                        log::debug!("exited pid: {:?}, status: {:?}", pid, status);
                        exit(status);
                    }
                    WaitStatus::Signaled(pid, status, _) => {
                        log::debug!("signaled pid: {:?}, status: {:?}", pid, status);
                        exit(0);
                    }
                    _ => bail!("abnormal exited!"),
                }
            }
        }
    }
}
