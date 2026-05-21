pub use super::common::Args;
use crate::ffi::OsString;
use crate::ptr;
use crate::sync::OnceLock;
use crate::sys::pal::abi;
use crate::sys::{FromInner, os_str};

pub fn args() -> Args {
    Args::new(ARGS.get_or_init(get_args).clone())
}

fn get_args() -> Vec<OsString> {
    let argc = unsafe { abi::sys_argc() };
    let mut args = Vec::with_capacity(argc);

    for i in 0..argc {
        // Get the size of the argument then the data.
        let nbytes = unsafe { abi::sys_argv(ptr::null_mut(), 0, i) };
        let mut buf = vec![0u8; nbytes];
        let nbytes2 = unsafe { abi::sys_argv(buf.as_mut_ptr(), buf.len(), i) };
        // The host may return a length that differs from the first call. Trust only what
        // was actually written and clamp to the buffer size.
        buf.truncate(core::cmp::min(nbytes2, buf.len()));
        args.push(OsString::from_inner(os_str::Buf { inner: buf }));
    }
    args
}

static ARGS: OnceLock<Vec<OsString>> = OnceLock::new();
