use super::error::Error;
use super::result::Result;
use super::su;
use super::sys;
use super::Tag;

use std::ffi::CStr;

use std::convert::TryFrom;

type EventClosure = dyn Fn(&mut Nua, Event, u32, String) + 'static;

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
            .finish()
    }
}

pub struct NuaBuilder {
    root: Option<su::Root>,
    tags: Vec<Tag>,
    closure: Option<Box<EventClosure>>,
}

impl std::fmt::Debug for NuaBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("NuaBuilder")
            .field("root", &self.root)
            .field("tags", &self.tags)
            .finish()
    }
}

impl NuaBuilder {
    pub fn default() -> Self {
        NuaBuilder {
            root: None,
            tags: Vec::<Tag>::new(),
            closure: None,
        }
    }
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn callback<F: Fn(&mut Nua, Event, u32, String) + 'static>(mut self, cb: F) -> Self {
        self.closure = Some(Box::new(cb));
        self
    }

    pub fn root(mut self, root: su::Root) -> Self {
        self.root = Some(root);
        self
    }

    pub fn create(self) -> Result<Box<Nua>> {
        let mut nua = Box::new(Nua::_new());
        let nua_ptr = &mut *nua as *mut Nua as *mut sys::nua_magic_t;

        let c_root = match &self.root {
            Some(root) => root.c_ptr,
            _ => crate::su::get_default_root()?.c_ptr,
        };

        let mut sys_tags = Vec::<sys::tagi_t>::new();
        for tag in &self.tags {
            sys_tags.push(tag.item());
        }

        /* last tag must be TAG_END or TAG_NULL */
        let tag_null = Tag::Null;
        sys_tags.push(tag_null.item());

        let sys_tags = sys_tags.as_slice();

        let c_callback = nua_callback_glue;
        let magic = nua_ptr;
        nua.closure = self.closure;
        nua.c_ptr = Nua::_create(c_root, Some(c_callback), magic, Some(sys_tags))?;
        nua.root = self.root;
        Ok(nua)
    }
}
/* Incomplete:
perl -lane 'print if s/pub const (nua_event_e_(.*)): nua_event_e = (\d+);/$1,/'  $(find $PWD -name bindings.rs | head -n1)
*/
/* from: https://stackoverflow.com/a/57578431/1522342 */
macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<i32> for $name {
            type Error = Error;

            fn try_from(v: i32) -> Result<Self> {
                match v {
                    $(x if x == $name::$vname as i32 => Ok($name::$vname),)*
                    _ => Err(Error::InitError),
                }
            }
        }
    }
}
back_to_enum! {
    #[derive(Debug, Clone)]
    pub enum Event {
        IncomingError = sys::nua_event_e_nua_i_error as isize,
        IncomingInvite = sys::nua_event_e_nua_i_invite as isize,
        IncomingCancel = sys::nua_event_e_nua_i_cancel as isize,
        IncomingAck = sys::nua_event_e_nua_i_ack as isize,
        IncomingFork = sys::nua_event_e_nua_i_fork as isize,
        IncomingActive = sys::nua_event_e_nua_i_active as isize,
        IncomingTerminated = sys::nua_event_e_nua_i_terminated as isize,
        IncomingState = sys::nua_event_e_nua_i_state as isize,
        IncomingOutbound = sys::nua_event_e_nua_i_outbound as isize,
        IncomingBye = sys::nua_event_e_nua_i_bye as isize,
        IncomingOptions = sys::nua_event_e_nua_i_options as isize,
        IncomingRefer = sys::nua_event_e_nua_i_refer as isize,
        IncomingPublish = sys::nua_event_e_nua_i_publish as isize,
        IncomingPrack = sys::nua_event_e_nua_i_prack as isize,
        IncomingInfo = sys::nua_event_e_nua_i_info as isize,
        Incomingupdate = sys::nua_event_e_nua_i_update as isize,
        IncomingMessage = sys::nua_event_e_nua_i_message as isize,
        IncomingChat = sys::nua_event_e_nua_i_chat as isize,
        IncomingSubscribe = sys::nua_event_e_nua_i_subscribe as isize,
        IncomingSubscription = sys::nua_event_e_nua_i_subscription as isize,
        IncomingNotify = sys::nua_event_e_nua_i_notify as isize,
        IncomingMethod = sys::nua_event_e_nua_i_method as isize,
        IncomingMediaError = sys::nua_event_e_nua_i_media_error as isize,
        ReplySetParams = sys::nua_event_e_nua_r_set_params as isize,
        ReplyGetParams = sys::nua_event_e_nua_r_get_params as isize,
        ReplyShutdown = sys::nua_event_e_nua_r_shutdown as isize,
        ReplyNotifier = sys::nua_event_e_nua_r_notifier as isize,
        ReplyTerminate = sys::nua_event_e_nua_r_terminate as isize,
        ReplyAuthorize = sys::nua_event_e_nua_r_authorize as isize,
        ReplyRegister = sys::nua_event_e_nua_r_register as isize,
        ReplyUnregister = sys::nua_event_e_nua_r_unregister as isize,
        ReplyInvite = sys::nua_event_e_nua_r_invite as isize,
        ReplyCancel = sys::nua_event_e_nua_r_cancel as isize,
        ReplyBye = sys::nua_event_e_nua_r_bye as isize,
        ReplyOptions = sys::nua_event_e_nua_r_options as isize,
        ReplyRefer = sys::nua_event_e_nua_r_refer as isize,
        ReplyPublish = sys::nua_event_e_nua_r_publish as isize,
        ReplyUnpublish = sys::nua_event_e_nua_r_unpublish as isize,
        ReplyInfo = sys::nua_event_e_nua_r_info as isize,
        ReplyPrack = sys::nua_event_e_nua_r_prack as isize,
        ReplyUpdate = sys::nua_event_e_nua_r_update as isize,
        ReplyMessage = sys::nua_event_e_nua_r_message as isize,
        ReplyChat = sys::nua_event_e_nua_r_chat as isize,
        ReplySubscribe = sys::nua_event_e_nua_r_subscribe as isize,
        ReplyUnsubscribe = sys::nua_event_e_nua_r_unsubscribe as isize,
        ReplyNotify = sys::nua_event_e_nua_r_notify as isize,
        ReplyMethod = sys::nua_event_e_nua_r_method as isize,
        ReplyAuthenticate = sys::nua_event_e_nua_r_authenticate as isize,
        ReplyRedirect = sys::nua_event_e_nua_r_redirect as isize,
        ReplyDestroy = sys::nua_event_e_nua_r_destroy as isize,
        ReplyRespond = sys::nua_event_e_nua_r_respond as isize,
        ReplyNitRespond = sys::nua_event_e_nua_r_nit_respond as isize,
        ReplyAck = sys::nua_event_e_nua_r_ack as isize,
        IncomingNetworkChanged = sys::nua_event_e_nua_i_network_changed as isize,
        IncomingRegister = sys::nua_event_e_nua_i_register as isize,
    }
}

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
    dbg!(_event, _status, _phrase, _nua, _magic, _nh, _hmagic, _sip, _tags);

    /*
    Panics can happen pretty much anywhere in Rust code.
    So to be safe, we need to wrap our callback body inside catch_unwind.
    see: https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics
    */
    if let Err(e) = std::panic::catch_unwind(|| {
        /* This call is expect to not panic if sofia does not changes their api */
        /* Also, it can happen if memory is corrupted and the process must be aborted, anyway */
        let event: Event = Event::try_from(_event as i32).unwrap();
        dbg!(&event);
        let status = _status as u32;
        let phrase: String = unsafe { CStr::from_ptr(_phrase).to_string_lossy().into_owned() };
        // let sys_nua: _nua;
        let nua: *mut Nua = _magic as *mut Nua;

        unsafe {
            (*nua).on_sys_nua_event(event, status, phrase, nua);
        }
    }) {
        // Code here must be panic-free.
        eprintln!("PANIC!! while calling a callback from C: {:?}", e);
        // Abort is safe because it doesn't unwind.
        std::process::abort();
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

    #[test]
    #[serial]
    fn create_nua_with_default_root() {
        wrap(|| {
            let b = NuaBuilder::default();

            b.create().unwrap();
        });
    }

    #[test]
    #[serial]
    fn create_nua_with_custom_root() {
        wrap(|| {
            let root = su::Root::new().unwrap();

            let b = NuaBuilder::default();
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

            let b = NuaBuilder::default();
            let b = b.root(root);
            let b = b.tag(url);

            b.create().unwrap();
        })
    }

}