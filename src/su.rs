use super::error::{errno, Error, ERROR_NONE};
use super::result::Result;
use super::sys;

use std::sync::atomic::{AtomicBool, Ordering};
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static ROOT_INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut DEFAULT_ROOT: Option<Root> = None;

#[derive(Debug)]
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

    pub(crate) fn _destroy(&mut self) {
        unsafe {
            if self.c_ptr.is_null() {
                return;
            }
            self.run_until_end();
            sys::su_root_destroy(self.c_ptr);
            self.c_ptr = std::ptr::null_mut();
        };
    }

    pub fn step(&self, timeout: Option<i64>) -> i64 {
        let timeout = match timeout {
            Some(x) if x > 0 && x < 1000 => x,
            _ => 100,
        };
        assert!(!self.c_ptr.is_null());
        unsafe { sys::su_root_step(self.c_ptr, timeout) }
    }

    pub fn run_until_end(&self) {
        while self.step(Some(1)) >= 0 {}
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn run(&mut self) {
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

/***********/
/* SU CORE */
/***********/
/*
FIXME:  creation/destroy is not threadsafe.
Check if it was called from the main thread before initilize.
We could use a static mutex for all globals
see http://gtk-rs.org/docs/src/gtk/rt.rs.html#83 (v0.9.0)
*/
pub fn init() -> Result<()> { /* TODO: drop Result, not used */
    match is_initialized() {
        true => Ok(()),
        false => {
            unsafe {
                sys::su_init();
            }
            /* su_init calls su_home_threadsafe(NULL) which sets errno */
            errno();

            extern "C" fn _at_exit_su_deinit() {
                deinit()
            }

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

pub fn deinit() {
    if ! is_initialized() {
        return;
    }
    unsafe {
        sys::su_deinit();
    };
    /* deinit default */
    deinit_default_root();
    INITIALIZED.store(false, Ordering::Release);
}

/*****************/
/* DEFAULT ROOT  */
/*****************/
/*
FIXME: creation/destroy is not threadsafe.
Check if it was called from the main thread before initilize.
We could use a static mutex for all globals
see http://gtk-rs.org/docs/src/gtk/rt.rs.html#83 (v0.9.0)
*/
pub fn init_default_root() -> Result<()> {
    init()?;
    match is_default_root_initialized() {
        true => Ok(()),
        false => {
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
                deinit_default_root();
            }
            unsafe { DEFAULT_ROOT = Some(root) };
            unsafe { sys::atexit(Some(_at_exit_destroy_root)) };

            ROOT_INITIALIZED.store(true, Ordering::Release);
            Ok(())
        }
    }
}


/// Returns `true` if default root has been initialized.
#[inline]
pub fn is_default_root_initialized() -> bool {
    ROOT_INITIALIZED.load(Ordering::Acquire)
}

fn get_default_root_as_mut() -> Result<&'static mut Root> {
    init_default_root()?;
    let root = unsafe { DEFAULT_ROOT.as_mut().unwrap() };
    Ok(root)
}

pub fn get_default_root() -> Result<&'static mut Root> {
    get_default_root_as_mut()
}

pub(crate) fn deinit_default_root() {
    if ! is_default_root_initialized() {
        return;
    }
    get_default_root_as_mut().unwrap()._destroy();
    unsafe { DEFAULT_ROOT = None };
    ROOT_INITIALIZED.store(false, Ordering::Release);
}

/*************/
/* MAIN LOOP */
/*************/
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

#[cfg(test)]
pub(crate) mod tests {
    use serial_test::serial;
    use super::*;

    pub fn wrap(f: fn()) {
        /* manual deinit (tests do not run atexit) */
        if let Err(e) = std::panic::catch_unwind(|| {
            f();
            deinit();
        }) {
            deinit();
            panic!("{:?}", e);
        }
    }

    #[test]
    #[serial]
    fn su_init() {
        wrap(|| {
            assert_eq!(is_initialized(), false);
            init().unwrap();
            assert_eq!(is_initialized(), true);
        });
    }

    #[test]
    #[serial]
    fn su_init_default_root() {
        wrap(|| {
            assert_eq!(is_default_root_initialized(), false);
            init_default_root().unwrap();
            assert_eq!(is_default_root_initialized(), true);
        });
    }

    #[test]
    #[serial]
    fn create_root() {
        wrap(|| {
            Root::new().unwrap();
        });
    }

    #[test]
    #[serial]
    fn step_must_return_negative_meaning_value_no_steps_to_run() {
        wrap(|| {
            let root = Root::new().unwrap();
            assert_eq!(root.step(Some(1)), -1);
        });
    }
}