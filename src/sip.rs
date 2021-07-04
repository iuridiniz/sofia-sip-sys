use crate::sys;
// use std::convert::From;
use std::convert::Into;
use std::fmt;

use std::ffi::CStr;

type SipSubject = MsgGeneric;

#[derive(Default, Debug)]
pub struct MsgGeneric {
    exists: bool,
    string: String,
}

impl MsgGeneric {
    pub(crate) fn _from_sys(sys_msg: *const sys::msg_generic_t) -> Self {
        let mut msg = MsgGeneric::default();
        if sys_msg.is_null() {
            return msg;
        }
        let sys_msg = unsafe { *sys_msg };

        msg.exists = true;

        assert!(!sys_msg.g_string.is_null());
        msg.string = unsafe {
            CStr::from_ptr(sys_msg.g_string)
                .to_string_lossy()
                .into_owned()
        };

        msg
    }
}

impl fmt::Display for MsgGeneric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl Into<String> for &MsgGeneric {
    fn into(self) -> String {
        format!("{}", self.string)
    }
}

/**********************************/
#[derive(Default, Debug)]
pub struct SipAddr {
    exists: bool,
    display: String,
    url: String,
}

/// Convert an url to a String.
fn url_as_string(sys_url_ptr: *const sys::url_t) -> String {
    assert!(!sys_url_ptr.is_null());

    /* first read length of c string */
    let len = unsafe { sys::url_e(std::ptr::null_mut(), 0, sys_url_ptr) as usize };

    /* create a buf to store c string plus '\0' */
    let buf_len = len + 1;
    let mut buf: Vec<u8> = vec![0; buf_len];
    unsafe { sys::url_e(buf.as_mut_ptr() as *mut i8, buf_len as i32, sys_url_ptr) };
    String::from_utf8_lossy(&buf[..len]).to_string()
}

impl SipAddr {
    pub(crate) fn _from_sys(sys_addr: *const sys::sip_addr_s) -> Self {
        let mut addr = SipAddr::default();
        if sys_addr.is_null() {
            return addr;
        }
        let sys_addr = unsafe { *sys_addr };

        if !sys_addr.a_display.is_null() {
            addr.display = unsafe {
                CStr::from_ptr(sys_addr.a_display)
                    .to_string_lossy()
                    .into_owned()
            };
        }

        let sys_url_ptr: *const sys::url_t = &sys_addr.a_url[0];

        addr.url = url_as_string(sys_url_ptr);

        addr.exists = true;

        addr
    }

    pub fn display(&self) -> &String {
        &self.display
    }
    pub fn url(&self) -> &String {
        &self.url
    }
}

impl fmt::Display for SipAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r: String = self.into();
        write!(f, "{}", r)
    }
}

impl Into<String> for &SipAddr {
    fn into(self) -> String {
        if self.display.len() > 0 {
            format!("{} {}", self.display, self.url)
        } else {
            format!("{}", self.url)
        }
    }
}

/**********************************/
#[derive(Default, Debug)]
pub struct Sip {
    exists: bool,
    from: SipAddr,
    to: SipAddr,
    subject: SipSubject,
}
impl Sip {
    pub(crate) fn _from_sys(sys_sip: *const sys::sip_t) -> Self {
        let mut sip = Sip::default();
        if sys_sip.is_null() {
            return sip;
        }

        let sys_sip = unsafe { *sys_sip };

        sip.from = SipAddr::_from_sys(sys_sip.sip_from);
        sip.to = SipAddr::_from_sys(sys_sip.sip_to);

        sip.subject = SipSubject::_from_sys(sys_sip.sip_subject);

        sip.exists = true;
        sip
    }

    pub fn from(&self) -> &SipAddr {
        &self.from
    }

    pub fn to(&self) -> &SipAddr {
        &self.to
    }

    pub fn subject(&self) -> &MsgGeneric {
        &self.subject
    }
}
