use crate::error::{errno, Error, ERROR_NONE};
use crate::result::Result;
use crate::sys;

use std::sync::atomic::{AtomicBool, Ordering};
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static ROOT_INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut DEFAULT_ROOT: Option<Root> = None;

#[derive(Debug)]
pub struct Root {
    pub(crate) c_ptr: *mut sys::su_root_t,
    rushing: bool,
}

impl Root {
    pub fn new() -> Result<Root> {
        init()?;
        Root::_create()
    }
    fn _create() -> Result<Root> {
        let root: *mut sys::su_root_t = unsafe { sys::su_root_create(std::ptr::null_mut() as _) };

        if root.is_null() {
            return Err(Error::InitError);
        }

        /* TODO: use a cargo feature */
        /* Disable threads */
        unsafe { sys::su_root_threading(root, 0) };

        Ok(Root {
            c_ptr: root,
            rushing: false,
        })
    }

    pub(crate) fn _destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }
        /* run in order to process any shutdown? */
        // self.rush_until_next_timer();
        unsafe {
            sys::su_root_destroy(self.c_ptr);
        }
        self.c_ptr = std::ptr::null_mut();
    }

    pub fn step(&self, timeout: Option<i64>) -> i64 {
        let timeout = match timeout {
            Some(x) if x >= 0 && x < 1000 => x,
            _ => 100,
        };
        self._step(timeout)
    }

    #[inline]
    pub(crate) fn _step(&self, timeout: i64) -> i64 {
        assert!(!self.c_ptr.is_null());
        let root: *mut sys::su_root_t = self.c_ptr;
        unsafe { sys::su_root_step(root, timeout) }
    }

    pub fn sleep(&self, timeout: i64) -> i64 {
        self._sleep(timeout)
    }

    #[inline]
    pub(crate) fn _sleep(&self, timeout: i64) -> i64 {
        assert!(!self.c_ptr.is_null());
        let root: *mut sys::su_root_t = self.c_ptr;
        unsafe { sys::su_root_sleep(root, timeout) }
    }

    pub fn run(&self) {
        self._run()
    }

    #[inline]
    pub(crate) fn _run(&self) {
        assert!(!self.c_ptr.is_null());
        let root: *mut sys::su_root_t = self.c_ptr;
        unsafe { sys::su_root_run(root) }
    }

    pub fn r#break(&self) {
        self._break()
    }

    pub fn break_(&self) {
        self.r#break()
    }

    #[inline]
    pub(crate) fn _break(&self) {
        assert!(!self.c_ptr.is_null());
        let root: *mut sys::su_root_t = self.c_ptr;
        unsafe { sys::su_root_break(root) }
    }

}

/* extra functions (without C equivalent) */
impl Root {
    pub fn rush_until_next_timer(&self) {
        loop {
            let remaining = self.step(Some(1));
            // dbg!(remaining);
            if remaining <= 0 {
                break;
            }
        }
    }

    pub fn rush(&mut self) {
        if self.rushing {
            return;
        }
        self.rushing = true;

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
            if !self.rushing {
                break;
            }
        }
    }
    pub fn stop(&mut self) {
        self.rushing = false;
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
            let root = Root::_create()?;
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
    root.rush();
    //root.run();
    Ok(())
}

pub fn main_loop_quit() {
    if let Ok(root) = get_default_root_as_mut() {
        root.stop()
        //root.break();
    };
}

#[cfg(test)]
pub(crate) mod tests {
    use serial_test::serial;
    use super::*;

    /* FIXME: Won't fix

    If an error occurs in any test using this library, it could affect all others
    following tests, since that this library need to be inited and deinit correctly.

    So, we have to deinit it correctly before prossed to next test.
    Fix this by doing a teardown per test (`wrap` does this)

    Also, each test run from a different thread by default (see --test-threads on
    `cargo test -- --help`) and need to be thread safe or do not use threads.
    Fix this by doing a setup/teardown that runs only once for all tests
    (`serial` can mitigate this)
    */
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
    fn su_init() {wrap(|| {
        assert_eq!(is_initialized(), false);
        init().unwrap();
        assert_eq!(is_initialized(), true);
    })}

    #[test]
    #[serial]
    fn su_init_default_root() {wrap(|| {
        assert_eq!(is_default_root_initialized(), false);
        init_default_root().unwrap();
        assert_eq!(is_default_root_initialized(), true);
    })}

    #[test]
    #[serial]
    fn create_root() {wrap(|| {
        Root::new().unwrap();
    })}

    #[test]
    #[serial]
    fn step_must_return_negative_meaning_no_steps_to_run() {wrap(|| {
        let root = Root::new().unwrap();
        assert_eq!(root.step(Some(1)), -1);
    })}
}