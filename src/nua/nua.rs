use crate::error::Error;
use crate::result::Result;
use crate::su;
use crate::sys;

pub use crate::nua::builder::Builder;
pub use crate::nua::event::Event;
pub use crate::nua::event::EventClosure;
pub use crate::nua::handle::Handle;

pub struct Nua<'a> {
    pub(crate) root: Option<&'a su::Root>,
    pub(crate) c_ptr: *mut sys::nua_t,
    pub(crate) closure: Option<Box<dyn Fn(&mut Nua, Event, u32, String) + 'a>>,
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
            cb(nua_for_closure, event, status, phrase);
        }
    }

    pub fn callback<F: Fn(&mut Nua, Event, u32, String) + 'a>(&mut self, cb: F) {
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
}

impl<'a> Drop for Nua<'a> {
    fn drop(&mut self) {
        self.destroy()
    }
}
