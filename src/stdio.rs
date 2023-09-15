use std::os::unix::io::RawFd;

#[derive(Debug)]
pub struct FileDescriptor(RawFd);

impl From<RawFd> for FileDescriptor {
    fn from(fd: RawFd) -> Self {
        FileDescriptor(fd)
    }
}