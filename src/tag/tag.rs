// use crate::error::Error;
use crate::sys;
use std::ffi::CString;
// use std::convert::TryFrom;

macro_rules! tag_cstring {
    ($tag:ident, $func:ident) => {
        #[allow(non_snake_case)]
        pub fn $func(s: &str) -> Self {
            Tag::$tag(CString::new(s).expect("unexpected '\0' character"))
        }
    };
}

#[derive(Debug, Clone)]
pub enum Tag {
    _PlaceHolder(CString),
    _NuUrl(CString),
    _NuMUsername(CString),
    _NuMDisplay(CString),
    _SoaUserSdpStr(CString),
    _SipSubjectStr(CString),
    _SipContentTypeStr(CString),
    _SipPayloadStr(CString),
    _SipToStr(CString),
    Null,
    End,
}
impl Tag {
    pub(crate) fn symbol(&self) -> sys::tag_type_t {
        match self {
            Tag::_PlaceHolder(_) => std::ptr::null() as sys::tag_type_t,
            Tag::_NuUrl(_) => unsafe { sys::nutag_url.as_ptr() },
            Tag::_NuMUsername(_) => unsafe { sys::nutag_m_username.as_ptr() },
            Tag::_NuMDisplay(_) => unsafe { sys::nutag_m_display.as_ptr() },
            Tag::_SoaUserSdpStr(_) => unsafe { sys::soatag_user_sdp_str.as_ptr() },
            Tag::_SipSubjectStr(_) => unsafe { sys::siptag_subject_str.as_ptr() },
            Tag::_SipContentTypeStr(_) => unsafe { sys::siptag_content_type_str.as_ptr() },
            Tag::_SipPayloadStr(_) => unsafe { sys::siptag_payload_str.as_ptr() },
            Tag::_SipToStr(_) => unsafe { sys::siptag_to_str.as_ptr() },
            Tag::Null | Tag::End => std::ptr::null() as sys::tag_type_t,
        }
    }
    pub(crate) fn value(&self) -> sys::tag_value_t {
        match self {
            Tag::_PlaceHolder(cstring)
            | Tag::_NuUrl(cstring)
            | Tag::_NuMUsername(cstring)
            | Tag::_NuMDisplay(cstring)
            | Tag::_SoaUserSdpStr(cstring)
            | Tag::_SipSubjectStr(cstring)
            | Tag::_SipContentTypeStr(cstring)
            | Tag::_SipPayloadStr(cstring)
            | Tag::_SipToStr(cstring) => cstring.as_ptr() as sys::tag_value_t,
            Tag::Null | Tag::End => 0 as sys::tag_value_t,
        }
    }

    pub(crate) fn item(&self) -> sys::tagi_t {
        sys::tagi_t {
            t_value: self.value(),
            t_tag: self.symbol(),
        }
    }

    tag_cstring!(_NuUrl, NuUrl);
    tag_cstring!(_NuMUsername, NuMUsername);
    tag_cstring!(_NuMDisplay, NuMDisplay);
    tag_cstring!(_SoaUserSdpStr, SoaUserSdpStr);
    tag_cstring!(_SipSubjectStr, SipSubjectStr);
    tag_cstring!(_SipContentTypeStr, SipContentTypeStr);
    tag_cstring!(_SipPayloadStr, SipPayloadStr);
    tag_cstring!(_SipToStr, SipToStr);
    // #[allow(non_snake_case)]
    // pub fn NuUrl(url: &str) -> Self {
    //     Tag::_NuUrl(CString::new(url).expect("unexpected '\0' character"))
    // }
}

/// Convert a tag to a String.
pub(crate) fn tagi_t_as_string(sys_tagi_ptr: *const sys::tagi_t) -> String {
    assert!(!sys_tagi_ptr.is_null());
    /* first read length of c string */
    let len = unsafe { sys::t_snprintf(sys_tagi_ptr, std::ptr::null_mut(), 0) } as usize;

    /* create a buf to store c string plus '\0' */
    let buf_len: usize = len + 1;
    let mut buf: Vec<u8> = vec![0; buf_len];
    unsafe { sys::t_snprintf(sys_tagi_ptr, buf.as_mut_ptr() as *mut i8, buf_len as u64) };
    String::from_utf8_lossy(&buf[..len]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tagi_t_as_string_nu_m_display() {
        let tag = Tag::NuMDisplay("foo");
        let tagi = tag.item();
        assert_eq!(
            tagi_t_as_string(&tagi as *const sys::tagi_t),
            "nua::m_display: \"foo\""
        );
    }

    #[test]
    fn test_tagi_t_as_string_nu_m_username() {
        let tag = Tag::NuMUsername("foo bar");
        let tagi = tag.item();
        assert_eq!(
            tagi_t_as_string(&tagi as *const sys::tagi_t),
            "nua::m_username: \"foo bar\""
        );
    }

    #[test]
    fn test_tagi_t_as_string_nu_url() {
        let tag = Tag::NuUrl("800@localhost");
        let tagi = tag.item();
        assert_eq!(
            tagi_t_as_string(&tagi as *const sys::tagi_t),
            "nua::url: <800@localhost>"
        );
    }

    #[test]
    fn test_tagi_t_as_string_null() {
        let tag = Tag::Null;
        let tagi = tag.item();
        assert_eq!(
            tagi_t_as_string(&tagi as *const sys::tagi_t),
            "::tag_null: 0"
        );
    }
}
