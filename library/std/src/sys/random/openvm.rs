use crate::sys::pal::abi;

pub fn fill_bytes(bytes: &mut [u8]) {
    if bytes.is_empty() {
        return;
    }
    unsafe { abi::sys_rand(bytes.as_mut_ptr(), bytes.len()) };
}
