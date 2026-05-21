#[expect(dead_code)]
#[path = "unsupported.rs"]
mod unsupported_env;
pub use unsupported_env::{Env, env, setenv, unsetenv};

use crate::ffi::{OsStr, OsString};
use crate::sys::pal::abi;
use crate::sys::{FromInner, os_str};

pub fn getenv(varname: &OsStr) -> Option<OsString> {
    let varname = varname.as_encoded_bytes();
    let nbytes =
        unsafe { abi::sys_getenv(crate::ptr::null_mut(), 0, varname.as_ptr(), varname.len()) };
    if nbytes == usize::MAX {
        return None;
    }

    let mut buf = vec![0u8; nbytes];
    let nbytes2 =
        unsafe { abi::sys_getenv(buf.as_mut_ptr(), buf.len(), varname.as_ptr(), varname.len()) };
    // The host may return a length that differs from the first call (the ABI does not
    // guarantee stable lengths across calls). Trust only what was actually written and
    // clamp to the buffer size.
    buf.truncate(core::cmp::min(nbytes2, buf.len()));

    Some(OsString::from_inner(os_str::Buf { inner: buf }))
}
