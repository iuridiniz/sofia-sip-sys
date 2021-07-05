// use crate::error::Error;
use crate::result::Result;
use crate::sys;
use std::ffi::CString;
// use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Tag {
    // NuUrl(String),
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
            Tag::_NuUrl(cstring)
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

    #[allow(non_snake_case)]
    pub fn NuUrl(url: &str) -> Result<Self> {
        Ok(Tag::_NuUrl(CString::new(url)?))
    }

    #[allow(non_snake_case)]
    pub fn SipSubject(s: &str) -> Result<Self> {
        Ok(Tag::_SipSubjectStr(CString::new(s)?))
    }

    #[allow(non_snake_case)]
    pub fn SipContentType(s: &str) -> Result<Self> {
        Ok(Tag::_SipContentTypeStr(CString::new(s)?))
    }

    #[allow(non_snake_case)]
    pub fn SipPayloadString(s: &str) -> Result<Self> {
        Ok(Tag::_SipPayloadStr(CString::new(s)?))
    }

    #[allow(non_snake_case)]
    pub fn SipTo(s: &str) -> Result<Self> {
        Ok(Tag::_SipToStr(CString::new(s)?))
    }
}
