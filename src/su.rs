use crate::error::{errno, Error};
use crate::result::Result;
use crate::sys;

use std::sync::atomic::{AtomicBool, Ordering};
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static ROOT_INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut DEFAULT_ROOT: Option<Root> = None;

#[derive(Debug)]
pub struct Root {
    pub(crate) c_ptr: *mut sys::su_root_t,
    pub(crate) rushing: bool,
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

    pub(crate) fn destroy(&mut self) {
        /* run in order to process any remaining shutdown? */
        // self.rush_until_next_timer();
        self._destroy()
    }

    #[inline]
    pub(crate) fn _destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }
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
}

impl Drop for Root {
    fn drop(&mut self) {
        self.destroy()
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
pub fn init() -> Result<()> {
    /* TODO: drop Result, not used */
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

            INITIALIZED.store(true, Ordering::Relaxed);
            Ok(())
        }
    }
}

/// Returns `true` if SOFIA-SIP-SU has been initialized.
#[inline]
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

pub fn deinit() {
    if !is_initialized() {
        return;
    }
    unsafe {
        sys::su_deinit();
    };

    INITIALIZED.store(false, Ordering::Relaxed);
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
            assert!(unsafe { DEFAULT_ROOT.is_none() });
            unsafe { DEFAULT_ROOT = Some(root) };
            unsafe { sys::atexit(Some(_at_exit_destroy_root)) };

            ROOT_INITIALIZED.store(true, Ordering::Relaxed);
            Ok(())
        }
    }
}

/// Returns `true` if default root has been initialized.
#[inline]
pub fn is_default_root_initialized() -> bool {
    ROOT_INITIALIZED.load(Ordering::Relaxed)
}

fn get_default_root_as_mut() -> Result<&'static mut Root> {
    init_default_root()?;
    let root = unsafe { DEFAULT_ROOT.as_mut().unwrap() };
    Ok(root)
}

pub fn get_default_root() -> Result<&'static Root> {
    let root = get_default_root_as_mut()?;
    Ok(root)
}

pub(crate) fn deinit_default_root() {
    match is_default_root_initialized() {
        false => (),
        true => {
            get_default_root_as_mut().unwrap().destroy();
            unsafe { DEFAULT_ROOT = None };
            ROOT_INITIALIZED.store(false, Ordering::Relaxed);
        }
    }

    assert!(unsafe { DEFAULT_ROOT.is_none() });
}

/*************/
/* MAIN LOOP */
/*************/
pub fn main_loop_run() -> Result<()> {
    let root = get_default_root()?;
    root.run();
    Ok(())
}

pub fn main_loop_quit() {
    if let Ok(root) = get_default_root_as_mut() {
        root.break_()
    };
}

/***************/
/* TEST HELPER */
/***************/

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
#[cfg(test)]
pub(crate) fn wrap(f: fn()) {
    /* manual deinit (tests do not run atexit) */
    if let Err(e) = std::panic::catch_unwind(|| {
        init().unwrap();
        init_default_root().unwrap();
        f();
        deinit_default_root();
        deinit();
    }) {
        deinit_default_root();
        deinit();
        println!(
            "******************************************************\n\
             PANIC INSIDE WRAPPER\n\
             `#[adorn(wrap)]` may give a wrong line that panicked\n\
             ******************************************************\n"
        );
        std::panic::resume_unwind(e);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use adorn::adorn;
    use serial_test::serial;

    mod su {
        use super::*;
        #[test]
        #[serial]
        fn su_init() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );

            init().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized is true after call init()"
            );

            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false after call deinit()"
            );
        }

        #[test]
        #[serial]
        fn su_deinit_without_init() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );

            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized remains false after call deinit()"
            );
        }

        #[test]
        #[serial]
        fn su_init_duplicate_calls() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );

            init().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized is true after call init()"
            );

            init().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized remains true after call second call to init()"
            );

            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false after call deinit()"
            );

            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized reamins false after second call deinit()"
            );
        }

        #[test]
        #[serial]
        fn su_init_deinit_many_times() {
            assert_eq!(
                is_initialized(),
                false,
                "FIRST INIT: assert is_initialized is false at very start"
            );

            init().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "FIRST INIT: assert is_initialized is true after call init()"
            );

            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "FIRST INIT: assert is_initialized is false after call deinit()"
            );

            assert_eq!(
                is_initialized(),
                false,
                "SECOND INIT: assert is_initialized is false at second start"
            );

            init().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "SECOND INIT: assert is_initialized is true after call init()"
            );

            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "SECOND INIT: assert is_initialized is false after call deinit()"
            );
        }

        #[test]
        #[serial]
        fn su_init_full() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false at very start"
            );

            /* su_init() */
            init().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized is true after call init()"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false after call init()"
            );

            /* init_default_root() */
            init_default_root().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized remains true after call init_default_root()"
            );
            assert_eq!(
                is_default_root_initialized(),
                true,
                "assert is_default_root_initialized is true after call init_default_root()"
            );

            /* deinit_default_root() */
            deinit_default_root();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized remains true after call deinit_default_root()"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false after call deinit_default_root()"
            );

            /* deinit() */
            deinit();
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false after call deinit()"
            );

            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized remains false after call deinit()"
            );
        }

        #[test]
        #[serial]
        fn su_init_default_root() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false at very start"
            );

            /* init_default_root() */
            init_default_root().unwrap();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized remains true after call init_default_root()"
            );
            assert_eq!(
                is_default_root_initialized(),
                true,
                "assert is_default_root_initialized is true after call init_default_root()"
            );

            /* deinit_default_root() */
            deinit_default_root();
            assert_eq!(
                is_initialized(),
                true,
                "assert is_initialized remains true after call deinit_default_root()"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false after call deinit_default_root()"
            );

            /* manual deinit */
            deinit();
        }

        #[test]
        #[serial]
        fn su_deinit_default_root_without_init() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false at very start"
            );

            /* deinit_default_root() */
            deinit_default_root();
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized remains false after call deinit_default_root()"
            );
            assert_eq!(
            is_default_root_initialized(),
            false,
            "assert is_default_root_initialized reamains false after call deinit_default_root()"
        );
        }

        #[test]
        #[serial]
        fn test_wrap_helper() {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false at very start"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized is false at very start"
            );
            wrap(|| {
                assert_eq!(
                    is_initialized(),
                    true,
                    "assert is_initialized is true inside wrapped fn"
                );
                assert_eq!(
                    is_default_root_initialized(),
                    true,
                    "assert is_default_root_initialized is true inside wrapped fn"
                );
            });

            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false after wrap run without panic"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized after wrap run without panic"
            );
        }
    }

    #[test]
    #[serial]
    fn test_wrap_helper_that_panics() {
        assert_eq!(
            is_initialized(),
            false,
            "assert is_initialized is false at very start"
        );
        assert_eq!(
            is_default_root_initialized(),
            false,
            "assert is_default_root_initialized is false at very start"
        );
        if let Err(_) = std::panic::catch_unwind(|| {
            wrap(|| {
                panic!("Just panic!");
            });
        }) {
            assert_eq!(
                is_initialized(),
                false,
                "assert is_initialized is false after wrap run and panics"
            );
            assert_eq!(
                is_default_root_initialized(),
                false,
                "assert is_default_root_initialized after wrap run and panics"
            );
        } else {
            assert!(false, "The wrapped function must panic")
        };
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn create_root() {
        Root::new().unwrap();
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn step_must_return_negative_meaning_no_steps_to_run() {
        let root = Root::new().unwrap();
        assert_eq!(root.step(Some(1)), -1);
    }
}
