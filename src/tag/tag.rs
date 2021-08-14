use crate::sys;
use std::ffi::CString;

#[derive(Debug, Clone)]
pub(crate) enum TagItem {
    NotImplementedStr(CString),
    NuUrl(CString),
    NuMUsername(CString),
    NuMDisplay(CString),
    SoaUserSdpStr(CString),
    SipSubjectStr(CString),
    SipContentTypeStr(CString),
    SipPayloadStr(CString),
    SipToStr(CString),
    Null,
    End,
}

impl TagItem {
    pub(crate) fn symbol(&self) -> sys::tag_type_t {
        match self {
            TagItem::NotImplementedStr(_) => std::ptr::null() as sys::tag_type_t,
            TagItem::NuUrl(_) => unsafe { sys::nutag_url.as_ptr() },
            TagItem::NuMUsername(_) => unsafe { sys::nutag_m_username.as_ptr() },
            TagItem::NuMDisplay(_) => unsafe { sys::nutag_m_display.as_ptr() },
            TagItem::SoaUserSdpStr(_) => unsafe { sys::soatag_user_sdp_str.as_ptr() },
            TagItem::SipSubjectStr(_) => unsafe { sys::siptag_subject_str.as_ptr() },
            TagItem::SipContentTypeStr(_) => unsafe { sys::siptag_content_type_str.as_ptr() },
            TagItem::SipPayloadStr(_) => unsafe { sys::siptag_payload_str.as_ptr() },
            TagItem::SipToStr(_) => unsafe { sys::siptag_to_str.as_ptr() },
            TagItem::Null | TagItem::End => std::ptr::null() as sys::tag_type_t,
        }
    }
    pub(crate) fn value(&self) -> sys::tag_value_t {
        match self {
            TagItem::NotImplementedStr(cstring)
            | TagItem::NuUrl(cstring)
            | TagItem::NuMUsername(cstring)
            | TagItem::NuMDisplay(cstring)
            | TagItem::SoaUserSdpStr(cstring)
            | TagItem::SipSubjectStr(cstring)
            | TagItem::SipContentTypeStr(cstring)
            | TagItem::SipPayloadStr(cstring)
            | TagItem::SipToStr(cstring) => cstring.as_ptr() as sys::tag_value_t,
            TagItem::Null | TagItem::End => 0 as sys::tag_value_t,
        }
    }

    pub(crate) fn item(&self) -> sys::tagi_t {
        sys::tagi_t {
            t_value: self.value(),
            t_tag: self.symbol(),
        }
    }

    /// Convert tag item (sys::tagi_t) to a String
    pub(crate) fn _tagi_t_to_string(sys_tagi_ptr: *const sys::tagi_t) -> String {
        assert!(!sys_tagi_ptr.is_null());
        /* first read length of c string */
        let len = unsafe { sys::t_snprintf(sys_tagi_ptr, std::ptr::null_mut(), 0) } as usize;

        /* create a buf to store c string plus '\0' */
        let buf_len: usize = len + 1;
        let mut buf: Vec<u8> = vec![0; buf_len];
        unsafe { sys::t_snprintf(sys_tagi_ptr, buf.as_mut_ptr() as *mut i8, buf_len as u64) };
        String::from_utf8_lossy(&buf[..len]).to_string()
    }

    /// Convert tag to a String using sofia representation.
    #[allow(dead_code)]
    pub fn sofia_string(&self) -> String {
        let tagi = self.item();
        return Self::_tagi_t_to_string(&tagi);
    }
}

/// Convert a rust string to a C string ignoring if anything after '\0' if any.
fn string_to_cstring_lossy(s: &str) -> CString {
    let bytes = s.as_bytes();

    let mut max = bytes.len();

    if let Some(pos) = bytes.iter().position(|&x| x == 0) {
        max = pos;
    }
    unsafe { CString::from_vec_unchecked(bytes[..max].to_vec()) }
}

impl From<&Tag> for TagItem {
    fn from(tag: &Tag) -> Self {
        match tag {
            Tag::NotImplementedStr(v) => TagItem::NotImplementedStr(string_to_cstring_lossy(v)),
            Tag::NuUrl(v) => TagItem::NuUrl(string_to_cstring_lossy(v)),
            Tag::NuMUsername(v) => TagItem::NuMUsername(string_to_cstring_lossy(v)),
            Tag::NuMDisplay(v) => TagItem::NuMDisplay(string_to_cstring_lossy(v)),
            Tag::SoaUserSdpStr(v) => TagItem::SoaUserSdpStr(string_to_cstring_lossy(v)),
            Tag::SipSubjectStr(v) => TagItem::SipSubjectStr(string_to_cstring_lossy(v)),
            Tag::SipContentTypeStr(v) => TagItem::SipContentTypeStr(string_to_cstring_lossy(v)),
            Tag::SipPayloadStr(v) => TagItem::SipPayloadStr(string_to_cstring_lossy(v)),
            Tag::SipToStr(v) => TagItem::SipToStr(string_to_cstring_lossy(v)),
            Tag::Null => Self::Null,
            Tag::End => Self::End,
        }
    }
}

impl From<Tag> for TagItem {
    fn from(tag: Tag) -> Self {
        Self::from(&tag)
    }
}

/// Convert a list of sys::tagi_t to a String.
// pub(crate) fn tagi_t_list_as_string(lst: *const sys::tagi_t) -> String {
//     let mut output = String::new();
//     let mut lst = lst;
//     while !lst.is_null() {
//         let s = tagi_t_as_string(lst);
//         output.push_str(&s);
//         output.push_str("\n");
//         lst = sys::t_next(lst);
//     }
//     return output;
// }

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    NotImplementedStr(String),
    NuUrl(String),
    NuMUsername(String),
    NuMDisplay(String),
    SoaUserSdpStr(String),
    SipSubjectStr(String),
    SipContentTypeStr(String),
    SipPayloadStr(String),
    SipToStr(String),
    Null,
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sofia_string() {
        assert_eq!(TagItem::Null.sofia_string(), "::tag_null: 0");

        assert_eq!(
            TagItem::NuMDisplay(CString::new("foo").unwrap()).sofia_string(),
            "nua::m_display: \"foo\""
        );
        assert_eq!(
            TagItem::NuMUsername(CString::new("foo bar").unwrap()).sofia_string(),
            "nua::m_username: \"foo bar\""
        );
        assert_eq!(
            TagItem::NuUrl(CString::new("800@localhost").unwrap()).sofia_string(),
            "nua::url: <800@localhost>"
        );
    }

    #[test]
    fn test_value() {
        assert_eq!(TagItem::Null.value(), 0);
    }

    #[test]
    fn test_symbol() {
        assert_eq!(TagItem::Null.symbol(), std::ptr::null());
    }

    #[test]
    fn test_item() {
        let i = TagItem::Null.item();
        assert_eq!(i.t_tag, TagItem::Null.symbol());
        assert_eq!(i.t_value, 0);
    }

    #[test]
    fn test_convert_tag_to_tagitem() {
        let tag = Tag::NuUrl("800@localhost".to_string());
        let tag_item: TagItem = tag.into();

        assert_eq!(tag_item.sofia_string(), "nua::url: <800@localhost>");

        let tag = Tag::NuUrl("800@localhost\0garbage".to_string());
        let tag_item: TagItem = tag.into();

        assert_eq!(tag_item.sofia_string(), "nua::url: <800@localhost>");

        let tag = &Tag::NuUrl("800@localhost".to_string());
        let tag_item: TagItem = tag.into();

        assert_eq!(tag_item.sofia_string(), "nua::url: <800@localhost>");
    }
}
