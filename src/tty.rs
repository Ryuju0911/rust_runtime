use std::os::unix::fs::symlink;
use std::path::PathBuf;

use anyhow::{Result, bail};
use nix::errno::Errno;
use nix::fcntl;
use nix::sys::stat;
use nix::sys::socket;

use crate::stdio::FileDescriptor;

pub fn load_console_sockets(
    container_dir: &PathBuf,
    console_socket: &str,
) -> Result<(FileDescriptor, FileDescriptor)> {
    let csocket = "console-stdout";
    symlink(console_socket, container_dir.join(csocket))?;

    let mut csocketfd = socket::socket(
        socket::AddressFamily::Unix,
        socket::SockType::Stream,
        socket::SockFlag::empty(),
        None,
    )?;
    csocketfd = match socket::connect(
        csocketfd,
        &socket::SockAddr::Unix(socket::UnixAddr::new(&*csocket)?),
    ) {
        Err(e) => {
            if e != ::nix::Error::Sys(Errno::ENOENT) {
                bail!("failed to open {}", csocket);
            }
            -1
        }
        Ok(()) => csocketfd,
    };
    let console = "console";
    let consolefd = match fcntl::open(
        &*console,
        fcntl::OFlag::O_NOCTTY | fcntl::OFlag::O_RDWR,
        stat::Mode::empty(),
    ) {
        Err(e) => {
            if e != ::nix::Error::Sys(Errno::ENOENT) {
                bail!("failed to open {}", console);
            }
            -1
        }
        Ok(fd) => fd,
    };
    Ok((csocketfd.into(), consolefd.into()))
}
