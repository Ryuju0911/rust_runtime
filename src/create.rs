use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{bail, Result};
use clap::Parser;
use nix::fcntl;
use nix::sched;
use nix::sys::stat;
use nix::unistd;
use nix::unistd::{Gid, Uid};

use crate::container::{Container, ContainerStatus};
use crate::notify_socket::NotifyListener;
use crate::process::{fork, Process};
use crate::rootfs;
use crate::spec;
use crate::stdio::FileDescriptor;
use crate::tty;
use crate::utils;

#[derive(Parser, Debug)]
pub struct Create {
    #[clap(short, long)]
    pid_file: Option<String>,
    #[clap(short, long, default_value = ".")]
    bundle: PathBuf,
    #[clap(short, long)]
    console_socket: Option<String>,
    pub container_id: String,
}

impl Create {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        println!("{} is being created...", self.container_id);
        let container_dir = root_path.join(&self.container_id);
        if !container_dir.exists() {
            fs::create_dir(&container_dir).unwrap();
        } else {
            bail!("{} already exists", self.container_id);
        }

        unistd::chdir(&self.bundle)?;

        let spec = spec::Spec::load("config.json")?;

        let container_dir = fs::canonicalize(container_dir)?;
        unistd::chdir(&*container_dir)?;

        let mut container = Container::new(
            &self.container_id,
            ContainerStatus::Creating,
            None,
            self.bundle.to_str().unwrap(),
            &container_dir,
        )?;
        container.save()?;

        let mut notify_socket: NotifyListener = NotifyListener::new(&container_dir)?;

        let rootfs = fs::canonicalize(&spec.root.path)?;

        let (csocketfd, _consolefd) = {
            if let Some(console_socket) = &self.console_socket {
                let (csocketfd, consolefd) =
                    tty::load_console_sockets(&container_dir, console_socket)?;
                (Some(csocketfd), Some(consolefd))
            } else {
                (None, None)
            }
        };

        let process = run_container(
            self.pid_file.as_ref(),
            &mut notify_socket,
            &rootfs,
            &spec,
            csocketfd,
            &mut container,
        )?;
        if let Process::Parent(_) = process {
            println!("{} was successfully created", self.container_id);
            process::exit(0);
        }
        Ok(())
    }

}

fn run_container<P: AsRef<Path>>(
    pid_file: Option<P>,
    notify_socket: &mut NotifyListener,
    rootfs: &PathBuf,
    spec: &spec::Spec,
    csocketfd: Option<FileDescriptor>,
    container: &mut Container,
) -> Result<Process>{
    prctl::set_dumpable(false).unwrap();
    let linux = spec.linux.as_ref().unwrap();

    let mut cf = sched::CloneFlags::empty();
    let mut to_enter = Vec::new();
    for ns in &linux.namespaces {
        let space = sched::CloneFlags::from_bits_truncate(ns.typ as i32);
        if ns.path.is_empty() {
            cf |= space;
        } else {
            let fd = fcntl::open(&*ns.path, fcntl::OFlag::empty(), stat::Mode::empty()).unwrap();
            to_enter.push((space, fd));
        }
    }

    match fork::fork_first(
        pid_file,
        cf.contains(sched::CloneFlags::CLONE_NEWUSER),
        linux,
        container,
    )? {
        Process::Parent(parent) => Ok(Process::Parent(parent)),
        Process::Child(child) => {
            if let Some(csocketfd) = csocketfd {
                tty::ready(csocketfd)?;
            }

            // join namepsaces
            for &(space, fd) in &to_enter {
                sched::setns(fd, space)?;
                unistd::close(fd)?;
                if space == sched::CloneFlags::CLONE_NEWUSER {
                    setid(Uid::from_raw(0), Gid::from_raw(0))?;
                }
            }

            // unshare other namespaces
            sched::unshare(cf & !sched::CloneFlags::CLONE_NEWUSER)?;

            /*
			 * We fork again because of PID namespace, setns(2) or unshare(2) don't
			 * change the PID namespace of the calling process, because doing so
			 * would change the caller's idea of its own PID (as reported by getpid()),
			 * which would break many applications and libraries, so we must fork
			 * to actually enter the new PID namespace.
			 */
            match fork::fork_init(child)? {
                Process::Child(child) => Ok(Process::Child(child)),
                Process::Init(mut init) => {
                    futures::executor::block_on(rootfs::prepare_rootfs(
                        spec,
                        rootfs,
                        cf.contains(sched::CloneFlags::CLONE_NEWUSER),
                    ))?;
                    rootfs::pivot_rootfs(&*rootfs)?;

                    init.ready()?;

                    notify_socket.wait_for_container_start()?;

                    utils::do_exec(&spec.process.args[0], &spec.process.args)?;
                    container.set_status(ContainerStatus::Stopped);
                    container.save()?;

                    Ok(Process::Init(init))
                }
                Process::Parent(_) => unreachable!()
            }
        }
        _ => unreachable!(),
    }
}

fn setid(uid: Uid, gid: Gid) -> Result<()> {
    if let Err(e) = prctl::set_keep_capabilities(true) {
        bail!("set keep capabilities returned {}", e);
    };

    unistd::setresgid(gid, gid, gid)?;
    unistd::setresuid(uid, uid, uid)?;
    if let Err(e) = prctl::set_keep_capabilities(false) {
        bail!("set keep capabilities returned {}", e);
    };
    Ok(())
}
