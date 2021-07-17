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
    tag_cstring!(_SipSubjectStr, SipSubjectStr);
    tag_cstring!(_SipContentTypeStr, SipContentTypeStr);
    tag_cstring!(_SipPayloadStr, SipPayloadStr);
    tag_cstring!(_SipToStr, SipToStr);
    // #[allow(non_snake_case)]
    // pub fn NuUrl(url: &str) -> Self {
    //     Tag::_NuUrl(CString::new(url).expect("unexpected '\0' character"))
    // }
}
