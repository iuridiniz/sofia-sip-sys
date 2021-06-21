#[derive(Debug)]
pub enum Error {
    InitError,
}

// https://chromium.googlesource.com/chromiumos/docs/+/master/constants/errnos.md
pub const ERROR_NONE: i32 = 0;
// const ERROR_EFAULT: i32 = 14;
// const ERROR_EAFNOSUPPORT: i32 = 97;

trait GetAndResetErrno {
    fn get_and_reset(&self) -> i32;
}
impl GetAndResetErrno for errno::Errno {
    fn get_and_reset(&self) -> i32 {
        let old = self.0;
        errno::set_errno(errno::Errno(ERROR_NONE));
        old
    }
}

pub fn errno() -> i32 {
    errno::errno().get_and_reset()
}
