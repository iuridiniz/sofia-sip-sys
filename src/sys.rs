/* http://sofia-sip.sourceforge.net/refdocs/nua/index.html */
/* http://sofia-sip.sourceforge.net/refdocs/nua/nua_8h.html */
/* https://github.com/freeswitch/sofia-sip */
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unknown_lints)]
#![allow(deref_nullptr)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    // https://chromium.googlesource.com/chromiumos/docs/+/master/constants/errnos.md
    const ERROR_NONE: i32 = 0;
    const ERROR_EFAULT: i32 = 14;
    const ERROR_EAFNOSUPPORT: i32 = 97;
    // const ERROR_EADDRINUSE: i32 = 98;

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

    fn errno() -> i32 {
        errno::errno().get_and_reset()
    }

    /* FIXME:
    If an error occurs in any test bellow, it could affect all others test,
    since that this library need to be inited and deinit correctly.
    When a test fails, the code needed to deinit the library is not executed.
    A teardown must be configured, to always deinit the library like
    crate::su::tests::wrap
    */
    use crate::sys;
    use serial_test::serial;
    use std::ffi::CString;
    #[test]
    #[serial]
    fn test_nua_init_and_deinit_with_threads() {
        errno();
        unsafe {
            let null: *const std::os::raw::c_void = std::ptr::null();

            /* Application context structure */
            let opaque: [u8; 1] = [42];
            let opaque_ptr = opaque.as_ptr() as *mut sys::su_root_magic_t;

            let home: sys::su_home_t = [0u64; 3usize];
            let home_ptr = home.as_ptr() as *mut sys::su_home_t;

            /* initialize system utilities */
            assert_eq!(errno(), ERROR_NONE);
            sys::su_init();
            /* su_init calls su_home_threadsafe(NULL) which sets errno */
            assert_eq!(errno(), ERROR_EFAULT);

            /* initialize memory handling */
            sys::su_home_init(home_ptr);
            assert_eq!(errno(), ERROR_NONE);

            /* initialize root object */
            let root = sys::su_root_create(opaque_ptr);
            assert!(!root.is_null(), "root cannot be null");

            // sys::su_root_threading(root, 0);
            // assert_eq!(errno(), ERROR_NONE);

            extern "C" fn cb(
                _event: sys::nua_event_t,
                _status: ::std::os::raw::c_int,
                _phrase: *const ::std::os::raw::c_char,
                _nua: *mut sys::nua_t,
                _magic: *mut sys::nua_magic_t,
                _nh: *mut sys::nua_handle_t,
                _hmagic: *mut sys::nua_hmagic_t,
                _sip: *const sys::sip_t,
                _tags: *mut sys::tagi_t,
            ) {
                dbg!(_event, _status, _phrase, _nua, _magic, _nh, _hmagic, _sip, _tags);

                let opaque_ptr: *mut [u8; 1] = _magic as *mut [u8; 1];

                unsafe {
                    assert_eq!((*opaque_ptr)[0], 42);
                    (*opaque_ptr)[0] = 42 + 42;
                }
            }
            let nua = sys::nua_create(
                root,
                Some(cb),
                opaque_ptr,
                null as *const sys::tag_type_s,
                0 as isize,
            );
            assert!(!nua.is_null());

            sys::nua_shutdown(nua);

            /* enter main loop for processing of messages */
            let remaining = sys::su_root_step(root, 100);
            assert_eq!(remaining, 0);
            assert_eq!(errno(), ERROR_NONE);

            /* destroy nua */
            sys::nua_destroy(nua);

            /* deinit root object */
            sys::su_root_destroy(root);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize memory handling */
            sys::su_home_deinit(home_ptr);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize system utilities */
            sys::su_deinit();
            assert_eq!(errno(), ERROR_NONE);

            assert_eq!(opaque[0], 42 + 42);
        }
    }
    #[test]
    #[serial]
    fn test_nua_init_and_deinit_without_threads() {
        errno();
        unsafe {
            let null: *const std::os::raw::c_void = std::ptr::null();

            /* Application context structure */
            let opaque: [u8; 1] = [31];
            let opaque_ptr = opaque.as_ptr() as *mut sys::su_root_magic_t;

            let home: sys::su_home_t = [0u64; 3usize];
            let home_ptr = home.as_ptr() as *mut sys::su_home_t;

            /* initialize system utilities */
            assert_eq!(errno(), ERROR_NONE);
            sys::su_init();
            /* su_init calls su_home_threadsafe(NULL) which sets errno */
            assert_eq!(errno(), ERROR_EFAULT);

            /* initialize memory handling */
            sys::su_home_init(home_ptr);
            assert_eq!(errno(), ERROR_NONE);

            /* initialize root object */
            let root = sys::su_root_create(opaque_ptr);
            assert!(!root.is_null(), "root cannot be null");

            /* disable threads */
            sys::su_root_threading(root, 0);
            assert_eq!(errno(), ERROR_NONE);

            extern "C" fn cb(
                _event: sys::nua_event_t,
                _status: ::std::os::raw::c_int,
                _phrase: *const ::std::os::raw::c_char,
                _nua: *mut sys::nua_t,
                _magic: *mut sys::nua_magic_t,
                _nh: *mut sys::nua_handle_t,
                _hmagic: *mut sys::nua_hmagic_t,
                _sip: *const sys::sip_t,
                _tags: *mut sys::tagi_t,
            ) {
                dbg!(_event, _status, _phrase, _nua, _magic, _nh, _hmagic, _sip, _tags);

                let opaque_ptr: *mut [u8; 1] = _magic as *mut [u8; 1];

                unsafe {
                    assert_eq!((*opaque_ptr)[0], 31);
                    (*opaque_ptr)[0] = 31 + 31;
                }
            }
            let nua = sys::nua_create(
                root,
                Some(cb),
                opaque_ptr,
                null as *const sys::tag_type_s,
                0 as isize,
            );
            assert!(!nua.is_null());
            assert_eq!(errno(), ERROR_EAFNOSUPPORT, "ERROR_EAFNOSUPPORT");

            sys::nua_shutdown(nua);

            /* enter main loop for processing of messages */
            let remaining = sys::su_root_step(root, 100);
            assert_eq!(remaining, -1);
            assert_eq!(errno(), ERROR_NONE);

            /* destroy nua */
            sys::nua_destroy(nua);

            /* deinit root object */
            sys::su_root_destroy(root);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize memory handling */
            sys::su_home_deinit(home_ptr);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize system utilities */
            sys::su_deinit();
            assert_eq!(errno(), ERROR_NONE);

            assert_eq!(opaque[0], 31 + 31);
        }
    }

    #[test]
    #[serial]
    fn test_su_root_null() {
        errno();
        let null: *const std::os::raw::c_void = std::ptr::null();
        assert_eq!(errno(), ERROR_NONE);
        let remaining;
        unsafe {
            /* test call with NULL */
            remaining = sys::su_root_step(null as *mut sys::su_root_t, 1000);
        }
        assert_eq!(remaining, -1);
        assert_eq!(errno(), ERROR_EFAULT);
    }

    #[test]
    #[ignore]
    #[serial]
    fn test_extension_incomplete() {
        /* Test for EXTENSION
        A                    B
        |------EXTENSION---->|
        |<--------501--------| (method not recognized)
        |                    |
        |------EXTENSION---->|
        |<-------200---------| (method allowed, responded)
        |                    |
        */
        errno();
        unsafe {
            let null: *mut std::os::raw::c_void = std::ptr::null_mut();

            /* Application context structure */
            let opaque_type = [0];
            let opaque_type_ptr = opaque_type.as_ptr() as *mut sys::su_root_magic_t;

            let home: sys::su_home_t = [0u64; 3usize];
            let home_ptr = home.as_ptr() as *mut sys::su_home_t;

            /* initialize system utilities */
            assert_eq!(errno(), ERROR_NONE);
            sys::su_init();
            /* su_init calls su_home_threadsafe(NULL) which sets errno */
            assert_eq!(errno(), ERROR_EFAULT);

            /* initialize memory handling */
            sys::su_home_init(home_ptr);

            /* initialize root object */
            let root = sys::su_root_create(null);
            assert!(!root.is_null());
            assert_eq!(errno(), ERROR_NONE);

            /* disable threads */
            sys::su_root_threading(root, 0);
            assert_eq!(errno(), ERROR_NONE);
            extern "C" fn cb(
                _event: sys::nua_event_t,
                _status: ::std::os::raw::c_int,
                _phrase: *const ::std::os::raw::c_char,
                _nua: *mut sys::nua_t,
                _magic: *mut sys::nua_magic_t,
                _nh: *mut sys::nua_handle_t,
                _hmagic: *mut sys::nua_hmagic_t,
                _sip: *const sys::sip_t,
                _tags: *mut sys::tagi_t,
            ) {
                println!("Hello");
            }
            let nua = sys::nua_create(
                root,
                Some(cb),
                opaque_type_ptr,
                // sys::nutag_url.as_ptr(),
                null as *const sys::tag_type_s,
                0 as isize,
            );
            assert!(!nua.is_null());
            assert_eq!(errno(), ERROR_EAFNOSUPPORT, "ERROR_EAFNOSUPPORT");

            sys::nua_shutdown(nua);

            /* enter main loop for processing of messages () */
            let mut remaining;
            loop {
                remaining = sys::su_root_step(root, 100);
                // assert_eq!(remaining, 1000 - 100);
                assert_eq!(errno(), ERROR_NONE);
                if remaining <= 100 {
                    break;
                }
            }

            /* destroy nua */
            sys::nua_destroy(nua);

            /* deinit root object */
            sys::su_root_destroy(root);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize memory handling */
            sys::su_home_deinit(home_ptr);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize system utilities */
            sys::su_deinit();
            assert_eq!(errno(), ERROR_NONE);
        }
    }

    #[test]
    #[serial]
    fn send_message_to_myself() {
        /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
        /*
        A
        |-------------------\
        |<------MESSAGE-----/
        |-------------------\
        |<--------200-------/
        |
        */
        errno();
        unsafe {
            let null: *mut std::os::raw::c_void = std::ptr::null_mut();

            /* initialize system utilities */
            assert_eq!(errno(), ERROR_NONE);
            sys::su_init();
            /* su_init calls su_home_threadsafe(NULL) which sets errno */
            assert_eq!(errno(), ERROR_EFAULT);

            /* initialize root object */
            let root = sys::su_root_create(null);
            assert!(!root.is_null());
            assert_eq!(errno(), ERROR_NONE);

            /* disable threads */
            sys::su_root_threading(root, 0);
            assert_eq!(errno(), ERROR_NONE);

            /* my callback */
            extern "C" fn cb(
                _event: sys::nua_event_t,
                _status: ::std::os::raw::c_int,
                _phrase: *const ::std::os::raw::c_char,
                _nua: *mut sys::nua_t,
                _magic: *mut sys::nua_magic_t,
                _nh: *mut sys::nua_handle_t,
                _hmagic: *mut sys::nua_hmagic_t,
                _sip: *const sys::sip_t,
                _tags: *mut sys::tagi_t,
            ) {
                dbg!(_event, _status, _phrase, _nua, _magic, _nh, _hmagic, _sip, _tags);

                match _event {
                    sys::nua_event_e_nua_r_shutdown => {
                        println!("Answer to nua_shutdown()");
                    }
                    sys::nua_event_e_nua_i_message => {
                        println!("Incoming MESSAGE");
                    }
                    sys::nua_event_e_nua_r_message => {
                        println!("Answer to outgoing MESSAGE");
                        unsafe { sys::nua_shutdown(_nua) };
                    }
                    _ => {
                        println!("Unknown event");
                    }
                }
                println!("--------------------------------------");
            }
            let nutag_url = CString::new("sip:127.0.0.1:5088").unwrap();
            let nua = sys::nua_create(
                root,
                Some(cb),
                null,
                sys::nutag_url.as_ptr(),
                nutag_url.as_ptr() as sys::tag_value_t,
                null as *const sys::tag_type_s,
                0 as isize,
            );
            assert!(!nua.is_null());
            /* clear errno */
            errno();
            // assert_ne!(errno(), ERROR_EADDRINUSE, "ERROR_EADDRINUSE");

            let hl = sys::nua_handle(
                nua,
                null,
                sys::nutag_url.as_ptr(),
                nutag_url.as_ptr() as sys::tag_value_t,
                sys::siptag_to_str.as_ptr(),
                nutag_url.as_ptr() as sys::tag_value_t,
                null as *const sys::tag_type_s,
                0 as isize,
            );
            assert!(!hl.is_null());
            assert_eq!(errno(), ERROR_NONE);

            sys::nua_message(hl, null as *const sys::tag_type_s, 0 as isize);

            /* enter main loop for processing of messages (3) */
            sys::su_root_step(root, 0);
            sys::su_root_step(root, 0);
            sys::su_root_step(root, 0);

            /* destroy nua */
            sys::nua_destroy(nua);

            /* deinit root object */
            sys::su_root_destroy(root);
            assert_eq!(errno(), ERROR_NONE);

            /* deinitialize system utilities */
            sys::su_deinit();
            assert_eq!(errno(), ERROR_NONE);
        }
    }
}
