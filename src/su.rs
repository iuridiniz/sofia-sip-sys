use super::error::{errno, Error, ERROR_NONE};
use super::result::Result;
use super::sys;

use std::sync::atomic::{AtomicBool, Ordering};
static INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut DEFAULT_ROOT: Option<Root> = None;

pub struct Root {
    pub(crate) c_ptr: *mut sys::su_root_t,
    running: bool,
}

impl Root {
    pub fn new() -> Result<Root> {
        init()?;
        Root::_new()
    }
    fn _new() -> Result<Root> {
        let root: *mut sys::su_root_t = unsafe { sys::su_root_create(std::ptr::null_mut() as _) };

        if root.is_null() {
            return Err(Error::InitError);
        }

        /* disable threads */
        // unsafe { sys::su_root_threading(root, 0) };

        Ok(Root {
            c_ptr: root,
            running: false,
        })
    }

    fn _destroy(&mut self) {
        unsafe {
            assert!(!self.c_ptr.is_null());
            sys::su_root_destroy(self.c_ptr);
        };
    }

    fn step(&mut self, timeout: Option<i64>) -> i64 {
        let timeout = match timeout {
            Some(x) if x > 0 && x < 1000 => x,
            _ => 100,
        };

        unsafe { sys::su_root_step(self.c_ptr, timeout) }
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn run(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        loop {
            /* clear errno */
            errno();
            let remaining = self.step(None);

            if remaining < 0 {
                let err = errno();
                if err != ERROR_NONE {
                    break;
                }
            }
            if !self.running {
                break;
            }
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        self._destroy()
    }
}

/*
FIXME: creation is not threadsafe.
Check if it was called from the main thread before initilize.
We could use a static mutex for all globals
see http://gtk-rs.org/docs/src/gtk/rt.rs.html#83 (v0.9.0)
*/
pub fn init() -> Result<()> {
    match is_initialized() {
        true => Ok(()),
        false => {
            unsafe {
                sys::su_init();
            }
            /* su_init calls su_home_threadsafe(NULL) which sets errno */
            errno();

            let root = Root::_new()?;

            /*
            According to https://doc.rust-lang.org/std/keyword.static.html
            "Static items do not call drop at the end of the program."

            So, static objects are not destroyed after main. This behaviour can be
            easily shown by using valgrind.

            Calling libc::atexit could be used to free that objects.

            See also: https://stackoverflow.com/a/48733480/1522342
            */
            extern "C" fn _at_exit_destroy_root() {
                unsafe {
                    DEFAULT_ROOT.as_mut().unwrap()._destroy();
                    sys::su_deinit();
                };
            }
            unsafe { DEFAULT_ROOT = Some(root) };
            unsafe { sys::atexit(Some(_at_exit_destroy_root)) };

            INITIALIZED.store(true, Ordering::Release);
            Ok(())
        }
    }
}

/// Returns `true` if SOFIA-SIP-SU has been initialized.
#[inline]
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

fn get_default_root_as_mut() -> Result<&'static mut Root> {
    init()?;
    let root = unsafe { DEFAULT_ROOT.as_mut().unwrap() };
    Ok(root)
}

pub fn get_default_root() -> Result<&'static Root> {
    let root: &Root = get_default_root_as_mut()?;
    Ok(root)
}

pub fn main_loop_run() -> Result<()> {
    let root = get_default_root_as_mut()?;
    root.run();
    Ok(())
}

pub fn main_loop_quit() {
    if let Ok(root) = get_default_root_as_mut() {
        root.quit()
    };
}
