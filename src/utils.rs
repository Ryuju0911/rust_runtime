use std::ffi::CString;

use anyhow::Result;
use nix::unistd;

pub fn do_exec(path: &str, args: &[String]) -> Result<()> {
    let p = CString::new(path.to_string()).unwrap();
    let a: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.to_string()).unwrap_or_default())
        .collect();

    unistd::execvp(&p, &a)?;
    Ok(())
}
