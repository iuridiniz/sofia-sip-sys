/* http://sofia-sip.sourceforge.net/refdocs/nua/index.html */
/* http://sofia-sip.sourceforge.net/refdocs/nua/nua_8h.html */
/* https://github.com/freeswitch/sofia-sip */
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


#[cfg(test)]
mod tests {
    // https://chromium.googlesource.com/chromiumos/docs/+/master/constants/errnos.md
    const ERROR_NONE: i32 = 0;
    const ERROR_EFAULT: i32 = 14;
    const ERROR_EAFNOSUPPORT: i32 = 97;

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

    use crate::sys;
    #[test]
    fn test_nua_init_and_deinit() {
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
    fn test_nua_init_and_deinit_without_threads() {
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
    fn test_su_root_null() {
        let null: *const std::os::raw::c_void = std::ptr::null();
        /* test call with NULL */
        assert_eq!(errno(), ERROR_NONE);
        let remaining;
        unsafe {
            remaining = sys::su_root_step(null as *mut sys::su_root_t, 1000);
        }
        assert_eq!(remaining, -1);
        assert_eq!(errno(), ERROR_EFAULT);
    }

    #[test]
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
                null as *const sys::tag_type_s,
                0 as isize,
            );
            assert!(!nua.is_null());
            assert_eq!(errno(), ERROR_EAFNOSUPPORT, "ERROR_EAFNOSUPPORT");

            /* enter main loop for processing of messages () */
            let mut remaining = 0;
            loop {
                remaining = sys::su_root_step(root, 100);
                // assert_eq!(remaining, 1000 - 100);
                assert_eq!(errno(), ERROR_NONE);
                if remaining <= 100 {
                    break;
                }
            }
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
}
