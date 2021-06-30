pub mod builder;
pub mod event;
pub mod handle;

use crate::error::Error;
use crate::result::Result;
use crate::su;
use crate::sys;

pub use crate::nua::builder::Builder;
pub use crate::nua::event::Event;
pub use crate::nua::event::EventClosure;

pub struct Nua<'a> {
    pub(crate) root: Option<&'a su::Root>,
    pub(crate) c_ptr: *mut sys::nua_t,
    pub(crate) closure: Option<Box<EventClosure>>,
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

    fn on_sys_nua_event(&self, event: Event, status: u32, phrase: String, nua: *mut Nua) {
        let nua = unsafe { &mut *nua };
        if let Event::ReplyShutdown = event {
            if status >= 200 {
                nua.shutdown_completed = true;
            }
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

impl<'a> Drop for Nua<'a> {
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
    fn create_nua_with_default_root() {wrap(|| {
        let b = Builder::default();

        b.create().unwrap();
    })}

    #[test]
    #[serial]
    fn create_nua_with_custom_root() {wrap(|| {
        let root = su::Root::new().unwrap();

        let b = Builder::default();
        let b = b.root(&root);

        b.create().unwrap();
    })}


    #[test]
    #[serial]
    fn create_nua_with_custom_url() {wrap(|| {
        let url = Tag::NuUrl("sip:*:5080".to_string()).unwrap();

        let root = su::Root::new().unwrap();

        let b = Builder::default();
        let b = b.root(&root);
        let b = b.tag(url);

        b.create().unwrap();
    })}

    #[test]
    #[serial]
    fn create_two_nua_with_same_port() {wrap(|| {
        let url = Tag::NuUrl("sip:*:5080".to_string()).unwrap();

        let root = su::Root::new().unwrap();

        let b = Builder::default();
        let b = b.root(&root);
        let b = b.tag(url);

        let _nua_a = b.create().unwrap();

        let url = Tag::NuUrl("sip:*:5080".to_string()).unwrap();

        let root = su::Root::new().unwrap();

        let b = Builder::default();
        let b = b.root(&root);
        let b = b.tag(url);

        assert!(b.create().is_err());
    })}

    #[test]
    #[ignore]
    #[serial]
    fn test_nua_a_send_message_to_nua_b() {wrap(|| {
        /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */

        /*
        A                    B
        |-------MESSAGE----->|
        |<--------200--------| (method allowed, responded)
        |                    |
        */

        let root = su::Root::new().unwrap();

        let mut nua_a = {
            let url = Tag::NuUrl("sip:127.0.0.1:5080".to_string()).unwrap();
            Builder::default()
                .root(&root)
                .tag(url)
                .create().unwrap()
        };

        // let nua_b = {
        //     let url = Tag::NuUrl("sip:127.0.0.1:5081".to_string()).unwrap();
        //     Builder::default()
        //         .root(&root)
        //         .tag(url)
        //         .create().unwrap()
        // };

        nua_a.callback(|nua: &mut Nua, event: Event, status: u32, phrase: String|{
            dbg!(&nua, &event, &status, &phrase);

        });

        let url = "Joe User <sip:joe.user@localhost:5081;param=1>;tag=12345678".to_string();


        let handle = Builder::default()
            // .tag(Tag::SipTo(url.clone()).unwrap())
            // .tag(Tag::NuUrl(url.clone()).unwrap())
            .create_handle(&nua_a).unwrap();

        // dbg!(&handle);

        let tags = Builder::default()
            .tag(Tag::SipSubject("NUA".to_string()).unwrap())
            .tag(Tag::SipTo(url.clone()).unwrap())
            .tag(Tag::NuUrl(url.clone()).unwrap())
            .tag(Tag::SipContentType("text/plain".to_string()).unwrap())
            .tag(Tag::SipPayload("Hi\n".to_string()).unwrap())
            .create_tags();
        // dbg!(&tags);


        println!("BEFORE MESSAGE");
        handle.message(tags);
        println!("AFTER MESSAGE");

        root.sleep(100);
        println!("AFTER RUN");

        panic!("abort");


    })}

    #[test]
    // #[ignore]
    #[serial]
    fn send_message_to_myself() {wrap(|| {

        /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
        /*
        A
        |-------------------\
        |<------MESSAGE-----/
        |-------------------\
        |<--------200-------/
        |
        */

        let root = su::Root::new().unwrap();
        let url = std::rc::Rc::new("sip:127.0.0.1:9997");

        let mut nua = {
            let url = Tag::NuUrl(url.clone().to_string()).unwrap();
            Builder::default()
                .root(&root)
                .tag(url)
                .create().unwrap()
        };

        nua.callback(|nua: &mut Nua, event: Event, status: u32, phrase: String|{
            dbg!(&nua, &event, &status, &phrase);
            // let root: &su::Root = nua.root();
            match event {
                Event::ReplyShutdown => {
                    // root.quit();
                },
                _ => {},
            }
        });


        let handle = Builder::default()
            .tag(Tag::SipTo(url.clone().to_string()).unwrap())
            .tag(Tag::NuUrl(url.clone().to_string()).unwrap())
            .create_handle(&nua).unwrap();

        // dbg!(&handle);

        let tags = Builder::default()
            .tag(Tag::SipSubject("NUA".to_string()).unwrap())
            .tag(Tag::SipTo(url.clone().to_string()).unwrap())
            .tag(Tag::NuUrl(url.clone().to_string()).unwrap())
            .tag(Tag::SipContentType("text/plain".to_string()).unwrap())
            .tag(Tag::SipPayload("Hi\n".to_string()).unwrap())
            .create_tags();

        handle.message(tags);
        root.sleep(1000);

        panic!("abort");
    })}

    #[test]
    #[ignore]
    #[serial]
    fn send_register_to_myself() {wrap(|| {



    })}

}