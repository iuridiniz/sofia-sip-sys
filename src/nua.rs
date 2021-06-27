// use super::error::errno;
// use super::error::Error;
use super::error::Error;
use super::result::Result;
use super::su;
use super::sys;
use super::Url;

use std::ffi::CStr;

use std::convert::TryFrom;

type MessagesCallback = fn();

#[derive(optargs::OptStruct)]
pub struct NuaTags {
    pub url: Option<Url>,
}

impl NuaTags {
    pub(crate) fn to_tag_list<'a>(&'a self) -> Vec<sys::tagi_t> {
        let mut tags = vec![];
        if let Some(url) = &self.url {
            // let t_value = url.t_value();
            // let t_tag = url.t_tag();
            // let tag = sys::tagi_t {
            //     // t_tag: unsafe { sys::nutag_url.as_ptr() },
            //     // t_value: c_str as *const _ as sys::tag_value_t,
            //     t_value: t_value,
            //     t_tag: t_tag,
            // };
            // // let bar = CString::new("sip:*:5070").unwrap();
            // // let tag = sys::tagi_t {
            // //     t_tag: unsafe { sys::nutag_url.as_ptr() as *const sys::tag_type_s },
            // //     t_value: bar.as_ptr() as isize,
            // // };
            tags.push(url.t_tagi());
        }
        tags
    }
}

pub struct Nua<'a> {
    messages_callback: Option<MessagesCallback>,
    root: &'a su::Root,
    pub(crate) c_ptr: *mut sys::nua_t,
    create_tags: crate::Tags,
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

extern "C" fn nua_app_callback_glue(
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
    So to be safe, we need to wrap our callback body inside catch_unwind
    */
    if let Err(e) = std::panic::catch_unwind(|| {
        let event: Event = Event::try_from(_event as i32).unwrap();
        let status = _status as u32;
        let phrase: String = unsafe { CStr::from_ptr(_phrase).to_string_lossy().into_owned() };
        // let sys_nua: _nua;
        let nua: *mut Nua = _magic as *mut Nua;

        unsafe {
            (*nua).app_callback(event, status, &phrase);
        }
    }) {
        // Code here must be panic-free.
        eprintln!("PANIC!! while calling a callback from C: {:?}", e);
        // Abort is safe because it doesn't unwind.
        std::process::abort();
    }
}

impl<'a> Nua<'a> {
    pub fn new(tags: crate::Tags) -> Result<Nua<'a>> {
        Nua::new_with_root(su::get_default_root()?, tags)
    }

    pub fn new_with_root(root: &'a su::Root, tags: crate::Tags) -> Result<Nua<'a>> {
        let nua = Nua {
            messages_callback: None,
            root: root,
            c_ptr: std::ptr::null_mut(),
            create_tags: tags,
        };

        Ok(nua)
    }

    pub fn messages_connect(&mut self, cb: MessagesCallback) {
        self._create();
        self.messages_callback = Some(cb);
        // unsafe { sys::nua_shutdown(self.c_ptr) };
    }

    fn app_callback(&mut self, event: Event, status: u32, phrase: &str) {
        println!(
            "Callback event: {:?} // status: {:?} // phrase: {:?}",
            event, status, phrase
        );
        if let Some(cb) = self.messages_callback {
            cb()
        } else {
            println!("callback is None")
        }
    }

    fn _create(&mut self) {
        if !self.c_ptr.is_null() {
            return;
        }
        let nua_ptr = self as *mut Nua as *mut sys::nua_magic_t;

        // let bar = CString::new("sip:*:5070").unwrap();
        // let tag_url = sys::tagi_t {
        //     t_tag: unsafe { sys::nutag_url.as_ptr() as *const sys::tag_type_s },
        //     t_value: bar.as_ptr() as isize,
        // };

        // let tag_null = sys::tagi_t {
        //     t_tag: std::ptr::null() as *const sys::tag_type_s,
        //     t_value: 0 as isize,
        // };
        let tags = self.create_tags.to_tag_list();
        let tags = tags.as_slice();

        // dbg!(a);
        // dbg!(b);

        let nua_sys = unsafe {
            sys::nua_create(
                self.root.c_ptr,
                Some(nua_app_callback_glue),
                nua_ptr,
                // sys::tag_next.as_ptr() as *const sys::tag_type_s,
                // self.create_tags.as_slice().as_ptr() as isize,
                // sys::nutag_url.as_ptr() as *const sys::tag_type_s,
                // bar.as_ptr() as isize,
                sys::tag_next.as_ptr() as *const sys::tag_type_s,
                tags.as_ptr() as isize,
                // self.create_tags.as_slice().as_ptr() as isize,
                // std::ptr::null() as *const sys::tag_type_s,
                // 0 as isize,
                // tag_url.t_tag,
                // tag_url.t_value,
                // tag_null.t_tag,
                // tag_null.t_value,
            )
        };
        dbg!(nua_sys);

        self.c_ptr = nua_sys;
    }

    fn _destroy(&mut self) {
        unsafe {
            sys::nua_destroy(self.c_ptr);
        }
    }
}

impl<'a> Drop for Nua<'a> {
    fn drop(&mut self) {
        self._destroy()
    }
}
