use std::os::fd::AsRawFd;
use std::os::raw::c_int;
use std::os::unix::net::UnixStream;

unsafe extern "C" {
    fn getuid() -> u32;
    fn getpeereid(socket: c_int, effective_uid: *mut u32, effective_gid: *mut u32) -> c_int;
}

pub(super) fn current_uid() -> u32 {
    unsafe { getuid() }
}

pub(super) fn peer_uid(stream: &UnixStream) -> Result<u32, ()> {
    let mut uid = 0;
    let mut gid = 0;
    let result = unsafe { getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) };
    (result == 0).then_some(uid).ok_or(())
}
