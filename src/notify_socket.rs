use std::os::unix::net::UnixListener;
use std::path::PathBuf;

use anyhow::Result;

pub const NOTIFY_FILE: &str = "notify.sock";

pub struct NotifyListener {
    socket: UnixListener,
}

impl NotifyListener {
    pub fn new(root: &PathBuf) -> Result<Self> {
        let _notify_file_path = root.join(NOTIFY_FILE);
        let stream = UnixListener::bind("notify.sock")?;
        Ok(Self { socket: stream })
    }
}
