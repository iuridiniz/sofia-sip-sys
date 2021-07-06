use crate::error::Error;
use crate::result::Result;
use crate::su;
use crate::su::Root;
use crate::sys;

pub use crate::nua::event::Event;
pub use crate::nua::event::EventClosure;
pub use crate::nua::handle::Handle;
use crate::sip::Sip;
// use crate::tag::builder::Builder;
use crate::tag::builder::convert_tags;
use crate::tag::Tag;

use std::convert::TryFrom;
use std::ffi::CStr;

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

    // pub fn create_nua(self) -> Result<Box<Nua>> {
    //     let mut nua = Box::new(Nua::_new());
    //     let nua_ptr = &mut *nua as *mut Nua as *mut sys::nua_magic_t;

    //     let c_root = match &self.root {
    //         Some(root) => root.c_ptr,
    //         _ => crate::su::get_default_root()?.c_ptr,
    //     };

    //     let tags = convert_tags(&self.tags);
    //     let sys_tags = tags.as_slice();

    //     let c_callback = nua_callback_glue;
    //     let magic = nua_ptr;
    //     nua.closure = self.closure;
    //     nua.c_ptr = Nua::_create(c_root, Some(c_callback), magic, Some(sys_tags))?;
    //     nua.root = self.root;
    //     Ok(nua)
    // }

    pub fn create(tags: Vec<Tag>) -> Result<Box<Nua<'a>>> {
        let root = crate::su::get_default_root()?;
        Self::create_with_root(root, tags)
    }

    pub fn create_with_root(root: &'a Root, tags: Vec<Tag>) -> Result<Box<Nua<'a>>> {
        let mut nua = Box::new(Nua::_new());
        let nua_ptr = &mut *nua as *mut Nua as *mut sys::nua_magic_t;

        let c_root = root.c_ptr;

        let tags = convert_tags(&tags);
        let sys_tags = tags.as_slice();

        let c_callback = nua_callback_glue;
        let magic = nua_ptr;
        // match closure {
        //     Some(cb) => {
        //         nua.callback(cb);
        //     }
        //     _ => {}
        // }
        nua.c_ptr = Self::_create(c_root, Some(c_callback), magic, Some(sys_tags))?;
        nua.root = Some(root);
        Ok(nua)
    }

    pub fn create_full<F: Fn(&mut Nua, Event, u32, String, Option<&Handle>, Sip, Vec<Tag>) + 'a>(
        root: &'a Root,
        closure: F,
        tags: Vec<Tag>,
    ) -> Result<Box<Nua<'a>>> {
        // let root = crate::su::get_default_root()?;
        let mut nua = Self::create_with_root(root, tags).unwrap();
        nua.callback(closure);
        Ok(nua)
    }

    pub fn root(&self) -> &su::Root {
        match &self.root {
            Some(root) => root,
            None => crate::su::get_default_root().unwrap(),
        }
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

    pub fn callback<F: Fn(&mut Nua, Event, u32, String, Option<&Handle>, Sip, Vec<Tag>) + 'a>(
        &mut self,
        cb: F,
    ) {
        self.closure = Some(Box::new(cb));
    }

    pub fn shutdown_and_wait(&self) {
        if self.shutdown_completed {
            return;
        }

        self.shutdown();
        while !self.shutdown_completed {
            if self.root().step(Some(1)) < 0 {
                break;
            }
        }
    }

    pub fn shutdown(&self) {
        if self.shutdown_completed {
            return;
        }
        self._shutdown();
    }

    fn _shutdown(&self) {
        assert!(!self.c_ptr.is_null());
        unsafe { sys::nua_shutdown(self.c_ptr) };
    }

    pub(crate) fn destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }
        /* before destroy we need to shutdown and wait for that shutdown */
        self.shutdown_and_wait();
        self._destroy();
    }

    fn _destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }

        unsafe {
            sys::nua_destroy(self.c_ptr);
        };
        self.c_ptr = std::ptr::null_mut();
    }

    pub fn run(&self) {
        self.root.unwrap().run();
    }

    pub fn r#break(&self) {
        self.root.unwrap().r#break();
    }

    pub fn break_(&self) {
        self.r#break();
    }

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

    // dbg!(_event, _status, _phrase, _nua, _magic, _nh, _hmagic, _sip, _tags);

    /*
    Panics can happen pretty much anywhere in Rust code.
    So to be safe, we need to wrap our callback body inside catch_unwind.
    see: https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics
    */
    if let Err(e) = std::panic::catch_unwind(|| {
        /* This call is expect to not panic if sofia does not changes their api */
        /* Also, it can happen if memory is corrupted and the process must be aborted, anyway */
        let event: Event = Event::try_from(_event as i32).unwrap();

        let status = _status as u32;

        let phrase: String = unsafe { CStr::from_ptr(_phrase).to_string_lossy().into_owned() };

        let sys_nua = _nua; /* ignored */

        let nua: *mut Nua = _magic as *mut Nua;

        /* sanity check */
        {
            assert!(!nua.is_null());
            let nua: &Nua = unsafe { &*nua };
            assert_eq!(sys_nua, nua.c_ptr);
        }

        let sys_handle = _nh; /* ignored */

        let handle: *mut Handle = _hmagic as *mut Handle;
        // let handle_struct: Option<&Handle>;
        if !handle.is_null() {
            /* reply to a owned handle function (outgoing sip message) */
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
        eprintln!("PANIC!! while calling a callback from C: {:?}", e);
        // Abort is safe because it doesn't unwind.
        std::process::abort();
    }
}
