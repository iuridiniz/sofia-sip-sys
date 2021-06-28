pub mod error;
pub mod nua;
pub mod result;
pub mod su;
pub mod sys;
pub mod tag;
pub use nua::Nua;
pub use nua::NuaBuilder;
pub use su::get_default_root;
pub use tag::Tag;

use std::convert::TryFrom;
use std::ffi::CString;

pub use nua::NuaTags;

#[derive(optargs::OptStruct)]
pub struct Tags {
    pub nua: Option<NuaTags>,
}

use std::sync::atomic::{AtomicBool, Ordering};
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() -> result::Result<()> {
    match is_initialized() {
        true => Ok(()),
        false => {
            su::init()?;
            INITIALIZED.store(true, Ordering::Release);
            Ok(())
        }
    }
}

/// Returns `true` if SOFIA-SIP has been initialized.
#[inline]
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

impl Tags {
    pub(crate) fn to_tag_list(&self) -> Vec<sys::tagi_t> {
        let mut tags = vec![];
        if let Some(nua_tags) = &self.nua {
            tags.append(&mut nua_tags.to_tag_list());
        }
        let tag_null = sys::tagi_t {
            t_tag: std::ptr::null() as *const sys::tag_type_s,
            t_value: 0 as isize,
        };
        tags.push(tag_null);
        tags
    }
}

pub struct Url {
    pub url: CString,
}

impl TryFrom<&str> for Url {
    type Error = error::Error;
    fn try_from(url: &str) -> result::Result<Self> {
        let url = CString::new(url)?;
        Ok(Url { url: url })
    }
}

impl TryFrom<String> for Url {
    type Error = error::Error;
    fn try_from(url: String) -> result::Result<Self> {
        let url = CString::new(url)?;
        Ok(Url { url: url })
    }
}
impl Url {
    pub(crate) fn t_value(&self) -> sys::tag_value_t {
        return self.url.as_ptr() as sys::tag_value_t;
    }

    pub(crate) fn t_tag() -> sys::tag_type_t {
        return unsafe { sys::nutag_url.as_ptr() };
    }

    pub(crate) fn t_tagi(&self) -> sys::tagi_t {
        return sys::tagi_t {
            t_value: self.t_value(),
            t_tag: Self::t_tag(),
        };
    }
}
