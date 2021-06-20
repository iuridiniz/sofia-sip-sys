use super::error::errno;
use super::error::Error;
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
        unsafe { sys::su_root_threading(root, 0) };

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

    fn run(&mut self) {
        use std::io::{self, Write};
        // let mut max = 0;
        if self.running {
            return;
        }
        self.running = true;
        loop {
            let remaining = unsafe { sys::su_root_step(self.c_ptr, 100) };
            // max += 1;
            // assert_eq!(remaining, 1000 - 100);
            // print!("[{}:{}]", remaining, errno());
            std::io::stdout().flush().unwrap();
            if remaining < 0 {
                break;
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
