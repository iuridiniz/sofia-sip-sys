use super::error::Error;
use super::result::Result;
use super::su;
use super::sys;
use super::Tag;
use super::Url;

use std::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;

use std::convert::TryFrom;

// type EventCallback = fn(&Nua, event: Event, status: u32, phrase: String);
type EventClosure = dyn Fn(&mut Nua, Event, u32, String) + 'static;

#[derive(optargs::OptStruct)]
pub struct NuaTags {
    pub url: Option<Url>,
}

impl NuaTags {
    pub(crate) fn to_tag_list<'a>(&'a self) -> Vec<sys::tagi_t> {
        let mut tags = vec![];
        if let Some(url) = &self.url {
            tags.push(url.t_tagi());
        }
        tags
    }
}

// #[derive(Clone)]
pub struct Nua {
    // pub(crate) callback: Option<EventCallback>, /* may be a vec of callbacks */
    pub(crate) c_ptr: *mut sys::nua_t,
    pub(crate) closure: Option<Box<EventClosure>>,
}

// impl Clone for Nua {
//     fn clone(&self) -> Self {
//         let nua = Nua::_new();
//         nua
//     }
// }

impl std::fmt::Debug for Nua {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Nua").field("c_ptr", &self.c_ptr).finish()
    }
}

// #[derive(Debug)]
pub struct NuaBuilder {
    root: Option<su::Root>,
    tags: Vec<Tag>,
    // callback: Option<EventCallback>, /* may be a vec of callbacks */
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
            // callback: None,
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
        // dbg!(nua_ptr);
        // dbg!(&nua as *const _);
        dbg!(&*nua as *const _);

        let root = match &self.root {
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
        // dbg!(sys_tags);

        let c_callback = nua_callback_glue;
        let magic = nua_ptr;
        nua.closure = self.closure;
        nua.c_ptr = Nua::_create(root, Some(c_callback), magic, Some(sys_tags))?;
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
        // dbg!(&event);
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
            // callback: None,
            closure: None,
            c_ptr: std::ptr::null_mut(),
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

        // unsafe { sys::nua_shutdown(nua_sys) }

        Ok(nua_sys)
    }

    fn on_sys_nua_event(&mut self, event: Event, status: u32, phrase: String, nua: *mut Nua) {
        // let nua: &Nua = unsafe { &*nua };
        // dbg!(&*self);
        if let Some(cb) = &self.closure {
            cb(unsafe { &mut *nua }, event, status, phrase);
        }
        Rc::new(self);
    }

    pub fn callback<F: Fn(&mut Nua, Event, u32, String) + 'static>(&mut self, cb: F) {
        self.closure = Some(Box::new(cb));
    }

    pub fn shutdown(&self) {
        unsafe { sys::nua_shutdown(self.c_ptr) }
    }

    fn _destroy(&mut self) {
        if self.c_ptr.is_null() {
            return;
        }
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
