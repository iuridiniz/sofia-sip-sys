use crate::error::Error;
use crate::result::Result;
use crate::su;
use crate::su::Root;
use crate::sys;

pub use crate::nua::event::Event;
pub use crate::nua::event::EventClosure;
pub use crate::nua::handle::Handle;
use crate::sip::Sip;
use crate::tag::builder::convert_tags;
use crate::tag::Tag;

use std::convert::TryFrom;
use std::ffi::CStr;

///NUA agent.
pub struct Nua<'a> {
    pub(crate) root: Option<&'a su::Root>,
    pub(crate) c_ptr: *mut sys::nua_t,
    pub(crate) closure:
        Option<Box<dyn Fn(&mut Nua, Event, u32, String, Option<&Handle>, Sip, Vec<Tag>) + 'a>>,
    shutdown_completed: bool,
}

impl<'a> std::fmt::Debug for Nua<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Nua")
            .field("Self", &(&*self as *const Nua))
            .field("c_ptr", &self.c_ptr)
            .field("root", &self.root)
            .finish()
    }
}

impl<'a> Nua<'a> {
    pub(crate) fn _new() -> Nua<'a> {
        Nua {
            root: None,
            closure: None,
            c_ptr: std::ptr::null_mut(),
            shutdown_completed: false,
        }
    }

    ///Create a NUA agent.
    pub fn create(tags: &[Tag]) -> Result<Box<Nua<'a>>> {
        let root = crate::su::get_default_root()?;
        Self::create_with_root(root, tags)
    }
    ///Create a NUA agent.
    pub fn create_with_root(root: &'a Root, tags: &[Tag]) -> Result<Box<Nua<'a>>> {
        let mut nua = Box::new(Nua::_new());
        let nua_ptr = &mut *nua as *mut Nua as *mut sys::nua_magic_t;

        let c_root = root.c_ptr;

        let tags = convert_tags(&tags);
        let sys_tags = tags.as_slice();

        let c_callback = nua_callback_glue;
        let magic = nua_ptr;

        nua.c_ptr = Self::_create(c_root, Some(c_callback), magic, Some(sys_tags))?;
        nua.root = Some(root);
        Ok(nua)
    }

    ///Create a NUA agent.
    pub fn create_full<F: Fn(&mut Nua, Event, u32, String, Option<&Handle>, Sip, Vec<Tag>) + 'a>(
        root: &'a Root,
        closure: F,
        tags: &[Tag],
    ) -> Result<Box<Nua<'a>>> {
        let mut nua = Self::create_with_root(root, tags).unwrap();
        nua.callback(closure);
        Ok(nua)
    }

    pub(crate) fn _create(
        root: *mut sys::su_root_t,
        callback: sys::nua_callback_f,
        magic: *mut sys::nua_magic_t,
        tags: Option<&[sys::tagi_t]>,
    ) -> Result<*mut sys::nua_s> {
        if root.is_null() {
            return Err(Error::CreateNuaError);
        }
        if callback.is_none() {
            return Err(Error::CreateNuaError);
        }
        if magic.is_null() {
            return Err(Error::CreateNuaError);
        }

        let tag_name: *const sys::tag_type_s;
        let tag_value: isize;

        if tags.is_none() {
            /* TAG_NULL */
            tag_name = std::ptr::null();
            tag_value = 0;
        } else {
            /* TAG_NEXT */
            tag_name = unsafe { sys::tag_next.as_ptr() };
            tag_value = tags.unwrap().as_ptr() as isize;
        }

        let nua_sys = unsafe { sys::nua_create(root, callback, magic, tag_name, tag_value) };

        if nua_sys.is_null() {
            /* failed to create */
            return Err(Error::CreateNuaError);
        }
        Ok(nua_sys)
    }

    pub(crate) fn _on_sys_nua_event(
        event: Event,
        status: u32,
        phrase: String,
        nua_ptr: *mut Nua,
        _handle_ptr: *mut Handle,
        sip: Sip,
        tags: Vec<Tag>,
    ) {
        assert!(!nua_ptr.is_null());
        let nua = unsafe { &mut *nua_ptr };
        match (&event, status) {
            (Event::ReplyShutdown, x) if x >= 200 => nua.shutdown_completed = true,
            (_, _) => {}
        }
        if let Some(cb) = &nua.closure {
            /* FIXME: not thread safe, we create a alias to a mutable Nua */
            let nua_for_closure = unsafe { &mut *nua_ptr };
            cb(nua_for_closure, event, status, phrase, None, sip, tags);
        }
    }

    ///Root reactor object.
    pub fn root(&self) -> &su::Root {
        match &self.root {
            Some(root) => root,
            None => crate::su::get_default_root().unwrap(),
        }
    }

    ///NUA event callback.
    pub fn callback<F: Fn(&mut Nua, Event, u32, String, Option<&Handle>, Sip, Vec<Tag>) + 'a>(
        &mut self,
        cb: F,
    ) {
        self.closure = Some(Box::new(cb));
    }

    ///Shutdown NUA stack.
    pub fn shutdown_and_wait(&self) {
        if self.shutdown_completed {
            return;
        }

        self.shutdown();
        while !self.shutdown_completed {
            if self.root().step0() < 0 {
                break;
            }
        }
    }

    ///Shutdown NUA stack.
    pub fn shutdown(&self) {
        if self.shutdown_completed {
            return;
        }
        Self::_shutdown(self.c_ptr);
    }

    pub(crate) fn _shutdown(nua: *mut sys::nua_s) {
        assert!(!nua.is_null());
        unsafe { sys::nua_shutdown(nua) };
    }

    /// Destroy the NUA stack.
    pub(crate) fn destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }
        /* before destroy we need to shutdown and wait for that shutdown */
        self.shutdown_and_wait();
        Self::_destroy(self.c_ptr);
        self.c_ptr = std::ptr::null_mut();
    }

    pub(crate) fn _destroy(nua: *mut sys::nua_s) {
        if nua.is_null() {
            return;
        }

        unsafe {
            sys::nua_destroy(nua);
        };
    }

    ///Run event and message loop.
    pub fn run(&self) {
        self.root.unwrap().run();
    }

    ///Terminate event loop.
    pub fn r#break(&self) {
        self.root.unwrap().r#break();
    }

    ///Terminate event loop.
    pub fn break_(&self) {
        self.r#break();
    }

    ///Terminate event loop.
    pub fn quit(&self) {
        self.r#break();
    }
}

impl<'a> Drop for Nua<'a> {
    fn drop(&mut self) {
        self.destroy()
    }
}

/// Called from C code, it will convert C types to Rust types and call Rust function with these types
extern "C" fn nua_callback_glue(
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
    // println!("------ nua_callback_glue ------");

    // dbg!(&_event, &_status, &_phrase, &_nua, &_magic, &_nh, &_hmagic, &_sip, &_tags);

    /*
    Panics can happen pretty much anywhere in Rust code.
    So to be safe, we need to wrap our callback body inside catch_unwind.
    see: https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics
    */
    if let Err(e) = std::panic::catch_unwind(|| {
        /* This call is expect to not panic if sofia does not changes their api */
        /* Also, it can happen if memory is corrupted and the process must be aborted, anyway */
        let event: Event = Event::try_from(_event as i32).unwrap();
        // dbg!(&event);
        let status = _status as u32;

        let phrase: String = unsafe { CStr::from_ptr(_phrase).to_string_lossy().into_owned() };

        let sys_nua = _nua; /* ignored */

        let nua: *mut Nua = _magic as *mut Nua;

        /* sanity check for Nua */
        {
            assert!(!nua.is_null());
            let nua: &Nua = unsafe { &*nua };
            assert_eq!(sys_nua, nua.c_ptr);
        }

        let sys_handle = _nh; /* ignored */

        let handle: *mut Handle = _hmagic as *mut Handle;
        // let handle_struct: Option<&Handle>;
        /* sanity check for Handle */
        if !handle.is_null() {
            /* reply to an owned handle function (outgoing sip message) */
            let handle: &Handle = unsafe { &*handle };
            assert_eq!(sys_handle, handle.c_ptr);
            // handle_struct = Some(handle_struct_temp);
        }

        let tags = Vec::<Tag>::new();

        // if !_tags.is_null() {
        // loop {
        // let t = unsafe { *_tags.offset(0) };
        // }
        // dbg!(t);
        // let mut v = Vec::<i8>::with_capacity(100);
        // let buf = v.as_mut_ptr();
        // unsafe { sys::t_snprintf(_tags.offset(0), buf, 100) };
        // let c_string = unsafe { std::ffi::CString::from_raw(buf) };
        // dbg!(&c_string);
        // /* avoid double free, by consuming c_string without dealloc */
        // c_string.into_raw(); // mem::forget(c_string)
        // }

        let sip = Sip::_from_sys(_sip);

        // println!("------ [nua_callback_glue] ------");
        Nua::_on_sys_nua_event(event, status, phrase, nua, handle, sip, tags);
    }) {
        // Code here must be panic-free.
        let error = format!("PANIC!! while calling a callback from C: {:?}\n\0", e);
        eprint!("{}", &error);
        // println!("{}", e);
        // unsafe { sys::printf(error.as_ptr() as *const i8) };
        // Abort is safe because it doesn't unwind.
        std::process::abort();
    }
}
#[cfg(test)]
mod tests {
    // use crate::Handle;
    // use crate::Nua;
    // use crate::Root;
    // use crate::Sip;
    // use crate::Tag;
    use crate::NuaEvent;
    use crate::TagBuilder;

    use super::*;

    use crate::su::wrap;
    use adorn::adorn;
    use serial_test::serial;

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn create_nua_with_default_root() {
        let tags = TagBuilder::default().collect();

        Nua::create(&tags).unwrap();
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn create_nua_with_custom_root() {
        let tags = TagBuilder::default().collect();
        let root = Root::create().unwrap();

        Nua::create_with_root(&root, &tags).unwrap();
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn nua_set_callback_to_closure() {
        let tags = TagBuilder::default().collect();
        let mut nua = Nua::create(&tags).unwrap();
        nua.callback(
            |nua: &mut Nua,
             event: NuaEvent,
             status: u32,
             phrase: String,
             handle: Option<&Handle>,
             sip: Sip,
             tags: Vec<Tag>| {
                dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
            },
        )
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn nua_set_callback_to_fn() {
        fn cb(
            nua: &mut Nua,
            event: NuaEvent,
            status: u32,
            phrase: String,
            handle: Option<&Handle>,
            sip: Sip,
            tags: Vec<Tag>,
        ) {
            dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
        }

        let tags = TagBuilder::default().collect();
        let mut nua = Nua::create(&tags).unwrap();
        nua.callback(cb);
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn create_nua_full() {
        fn cb(
            nua: &mut Nua,
            event: NuaEvent,
            status: u32,
            phrase: String,
            handle: Option<&Handle>,
            sip: Sip,
            tags: Vec<Tag>,
        ) {
            dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
        }

        let tags = TagBuilder::default().collect();
        let root = Root::create().unwrap();

        let mut nua = Nua::create_full(&root, cb, &tags).unwrap();
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn create_nua_with_custom_url() {
        let url = Tag::NuUrl("sip:*:5080").unwrap();

        let root = Root::create().unwrap();

        let tags = TagBuilder::default().tag(url).collect();

        Nua::create(&tags).unwrap();
    }

    #[test]
    #[adorn(wrap)]
    #[serial]
    fn create_two_nua_with_same_port() {
        let url = Tag::NuUrl("sip:*:5080").unwrap();

        let root = Root::create().unwrap();

        let b = TagBuilder::default();
        let b = b.tag(url);
        let tags = b.collect();

        let _nua_a = Nua::create_with_root(&root, &tags).unwrap();

        let url = Tag::NuUrl("sip:*:5080").unwrap();

        let root = Root::create().unwrap();

        let b = TagBuilder::default();
        let b = b.tag(url);
        let tags = b.collect();

        assert!(Nua::create_with_root(&root, &tags).is_err());
    }
}
