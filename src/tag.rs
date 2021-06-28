// use crate::error::Error;
use crate::result::Result;
use crate::sys;
use std::ffi::CString;
// use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Tag {
    // NuUrl(String),
    _NuUrl(CString),
    Null,
    End,
}
impl Tag {
    pub(crate) fn symbol(&self) -> sys::tag_type_t {
        match self {
            Tag::_NuUrl(_) => unsafe { sys::nutag_url.as_ptr() },
            Tag::Null | Tag::End => std::ptr::null() as sys::tag_type_t,
        }
    }
    pub(crate) fn value(&self) -> sys::tag_value_t {
        match self {
            Tag::_NuUrl(url) => url.as_ptr() as sys::tag_value_t,
            Tag::Null | Tag::End => 0 as sys::tag_value_t,
        }
    }

    pub(crate) fn item(&self) -> sys::tagi_t {
        sys::tagi_t {
            t_value: self.value(),
            t_tag: self.symbol(),
        }
    }
    // pub(crate) fn is_string(&self) -> bool {
    //     match self {
    //         Tag::NuUrl(_) => true,
    //         _ => false,
    //     }
    // }

    // pub fn convert(self) -> Result<Self> {
    //     match self {
    //         Tag::NuUrl(url) => Ok(Tag::_NuUrl(CString::new(url)?)),
    //         _ => Ok(self),
    //     }
    // }

    pub fn NuUrl(url: String) -> Result<Self> {
        Ok(Tag::_NuUrl(CString::new(url)?))
    }
}
