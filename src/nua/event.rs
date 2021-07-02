use crate::error::Error;
use crate::nua::nua::Handle;
use crate::nua::Nua;
use crate::result::Result;
use crate::sys;
use crate::tag::Tag;

pub type EventClosure =
    dyn Fn(&mut Nua, Event, u32, String, Option<&Handle>, Option<Vec<Tag>>) + 'static;

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
