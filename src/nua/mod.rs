pub mod builder;
pub mod event;

use crate::error::Error;
use crate::result::Result;
use crate::su;
use crate::sys;

pub use crate::nua::builder::Builder;
pub use crate::nua::event::Event;
pub use crate::nua::event::EventClosure;


pub struct Nua {
    pub(crate) root: Option<su::Root>,
    pub(crate) c_ptr: *mut sys::nua_t,
    pub(crate) closure: Option<Box<EventClosure>>,
    shutdown_completed: bool,
}

impl std::fmt::Debug for Nua {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Nua")
            .field("Self", &(&*self as *const Nua))
            .field("c_ptr", &self.c_ptr)
            .field("root", &self.root)
            .finish()
    }
}


impl Nua {
    pub(crate) fn _new() -> Nua {
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

    fn on_sys_nua_event(&self, event: Event, status: u32, phrase: String, nua: *mut Nua) {
        let nua = unsafe { &mut *nua };
        if let Event::ReplyShutdown = event {
            nua.shutdown_completed = true;
        }
        if let Some(cb) = &self.closure {
            cb(nua, event, status, phrase);
        }
    }

    pub fn callback<F: Fn(&mut Nua, Event, u32, String) + 'static>(&mut self, cb: F) {
        self.closure = Some(Box::new(cb));
    }

    pub fn shutdown(&self) {
        if self.shutdown_completed == false {
            unsafe { sys::nua_shutdown(self.c_ptr) };
        }
    }
    pub fn shutdown_and_wait(&self) {
        self.shutdown();
        while self.shutdown_completed == false {
            if self.root().step(Some(1)) < 0 {
                break;
            }
        }
    }

    fn _destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }

        /* before destroy we need to shutdown and wait for that shutdown */
        self.shutdown_and_wait();

        unsafe {
            sys::nua_destroy(self.c_ptr);
            self.c_ptr = std::ptr::null_mut();
        }
    }
}

impl Drop for Nua {
    fn drop(&mut self) {
        self._destroy()
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use super::*;
    use super::su::tests::wrap;
    use crate::Tag;


    #[test]
    #[serial]
    fn create_nua_with_default_root() {
        wrap(|| {
            let b = Builder::default();

            b.create().unwrap();
        });
    }

    #[test]
    #[serial]
    fn create_nua_with_custom_root() {
        wrap(|| {
            let root = su::Root::new().unwrap();

            let b = Builder::default();
            let b = b.root(root);

            b.create().unwrap();
        })
    }

    #[test]
    #[serial]
    fn create_nua_with_custom_url() {

        wrap(|| {
            let url = Tag::NuUrl("sip:*:5080".to_string()).unwrap();

            let root = su::Root::new().unwrap();

            let b = Builder::default();
            let b = b.root(root);
            let b = b.tag(url);

            b.create().unwrap();
        })
    }

}