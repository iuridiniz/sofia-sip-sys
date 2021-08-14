use crate::sys;
use std::ffi::CStr;
use std::ffi::CString;

#[derive(Debug, Clone, Default)]
pub struct TagTypeClass {}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TagType {
    namespace: CString,
    name: CString,
}

impl TagType {
    pub(crate) fn _from_sys(tt: *const sys::tag_type_s) -> Self {
        let mut tag_type = TagType::default();
        if tt.is_null() {
            return tag_type;
        }
        let tt: sys::tag_type_s = unsafe { *tt };
        if !tt.tt_ns.is_null() {
            let ns = unsafe { CStr::from_ptr(tt.tt_ns) };
            tag_type.namespace = ns.to_owned();
        }
        if !tt.tt_name.is_null() {
            let name = unsafe { CStr::from_ptr(tt.tt_name) };
            tag_type.name = name.to_owned();
        }

        tag_type
    }

    // pub(crate) fn symbol(&self) -> Option<sys::tag_type_t> {
    //     if self.namespace == "nua" {
    //         if self.name == "url" {
    //             return Some(unsafe { sys::nutag_url.as_ptr() });
    //         }
    //     }
    //     None
    // }
}

#[derive(Debug, Clone)]
pub(crate) enum TagItem {
    _PlaceHolder(CString),
    NuUrl(CString),
    NuMUsername(CString),
    NuMDisplay(CString),
    SoaUserSdpStr(CString),
    SipSubjectStr(CString),
    SipContentTypeStr(CString),
    SipPayloadStr(CString),
    SipToStr(CString),
    NotImplemented(TagType),
    Null,
    End,
}

impl TagItem {
    pub(crate) fn symbol(&self) -> sys::tag_type_t {
        match self {
            TagItem::NotImplemented(_) => std::ptr::null() as sys::tag_type_t,
            TagItem::_PlaceHolder(_) => std::ptr::null() as sys::tag_type_t,
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
            TagItem::_PlaceHolder(cstring)
            | TagItem::NuUrl(cstring)
            | TagItem::NuMUsername(cstring)
            | TagItem::NuMDisplay(cstring)
            | TagItem::SoaUserSdpStr(cstring)
            | TagItem::SipSubjectStr(cstring)
            | TagItem::SipContentTypeStr(cstring)
            | TagItem::SipPayloadStr(cstring)
            | TagItem::SipToStr(cstring) => cstring.as_ptr() as sys::tag_value_t,
            TagItem::NotImplemented(_) | TagItem::Null | TagItem::End => 0 as sys::tag_value_t,
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

    fn _convert_t_value_to_cstring(t_value: sys::tag_value_t) -> CString {
        if t_value == 0 {
            CString::new("").unwrap()
        } else {
            unsafe { CStr::from_ptr(t_value as *mut i8).to_owned() }
        }
    }

    pub(crate) fn _from_sys(tagi: *const sys::tagi_t) -> Self {
        if tagi.is_null() {
            return Self::Null;
        }
        let tagi: sys::tagi_t = unsafe { *tagi };
        let tag_type = tagi.t_tag;
        let tag_value = tagi.t_value;
        if tag_type.is_null() {
            return Self::Null;
        }
        // {
        //     let tag_type = TagType::_from_sys(tagi.t_tag);
        //     dbg!(tag_type);
        // }

        /* match symbol */
        unsafe {
            if tag_type == sys::tag_null.as_ptr() {
                Self::Null
            } else if tag_type == sys::nutag_url.as_ptr() {
                let url = Self::_convert_t_value_to_cstring(tag_value);
                Self::NuUrl(url)
            } else if tag_type == sys::nutag_m_username.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::NuMUsername(v)
            } else if tag_type == sys::nutag_m_display.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::NuMDisplay(v)
            } else if tag_type == sys::soatag_user_sdp_str.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::SoaUserSdpStr(v)
            } else if tag_type == sys::siptag_subject_str.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::SipSubjectStr(v)
            } else if tag_type == sys::siptag_content_type_str.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::SipContentTypeStr(v)
            } else if tag_type == sys::siptag_payload_str.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::SipPayloadStr(v)
            } else if tag_type == sys::siptag_to_str.as_ptr() {
                let v = Self::_convert_t_value_to_cstring(tag_value);
                Self::SipToStr(v)
            } else {
                let tag_type = TagType::_from_sys(tagi.t_tag);
                Self::NotImplemented(tag_type)
            }
        }
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
            Tag::_PlaceHolder(v) => TagItem::_PlaceHolder(string_to_cstring_lossy(v)),
            Tag::NuUrl(v) => TagItem::NuUrl(string_to_cstring_lossy(v)),
            Tag::NuMUsername(v) => TagItem::NuMUsername(string_to_cstring_lossy(v)),
            Tag::NuMDisplay(v) => TagItem::NuMDisplay(string_to_cstring_lossy(v)),
            Tag::SoaUserSdpStr(v) => TagItem::SoaUserSdpStr(string_to_cstring_lossy(v)),
            Tag::SipSubjectStr(v) => TagItem::SipSubjectStr(string_to_cstring_lossy(v)),
            Tag::SipContentTypeStr(v) => TagItem::SipContentTypeStr(string_to_cstring_lossy(v)),
            Tag::SipPayloadStr(v) => TagItem::SipPayloadStr(string_to_cstring_lossy(v)),
            Tag::SipToStr(v) => TagItem::SipToStr(string_to_cstring_lossy(v)),
            Tag::NotImplemented(v) => TagItem::NotImplemented(v.clone()),
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
    _PlaceHolder(String),
    NuUrl(String),
    NuMUsername(String),
    NuMDisplay(String),
    SoaUserSdpStr(String),
    SipSubjectStr(String),
    SipContentTypeStr(String),
    SipPayloadStr(String),
    SipToStr(String),
    NotImplemented(TagType),
    Null,
    End,
}

impl From<&TagItem> for Tag {
    fn from(tag: &TagItem) -> Self {
        match tag {
            TagItem::_PlaceHolder(v) => Tag::_PlaceHolder(v.to_string_lossy().into_owned()),
            TagItem::NuUrl(v) => Tag::NuUrl(v.to_string_lossy().into_owned()),
            TagItem::NuMUsername(v) => Tag::NuMUsername(v.to_string_lossy().into_owned()),
            TagItem::NuMDisplay(v) => Tag::NuMDisplay(v.to_string_lossy().into_owned()),
            TagItem::SoaUserSdpStr(v) => Tag::SoaUserSdpStr(v.to_string_lossy().into_owned()),
            TagItem::SipSubjectStr(v) => Tag::SipSubjectStr(v.to_string_lossy().into_owned()),
            TagItem::SipContentTypeStr(v) => {
                Tag::SipContentTypeStr(v.to_string_lossy().into_owned())
            }
            TagItem::SipPayloadStr(v) => Tag::SipPayloadStr(v.to_string_lossy().into_owned()),
            TagItem::SipToStr(v) => Tag::SipToStr(v.to_string_lossy().into_owned()),
            TagItem::NotImplemented(v) => Tag::NotImplemented(v.clone()),
            TagItem::Null => Self::Null,
            TagItem::End => Self::End,
        }
    }
}

impl From<TagItem> for Tag {
    fn from(tag: TagItem) -> Self {
        Self::from(&tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_item_sofia_string() {
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
    fn test_tag_item_value() {
        assert_eq!(TagItem::Null.value(), 0);
    }

    #[test]
    fn test_tag_item_symbol() {
        assert_eq!(TagItem::Null.symbol(), std::ptr::null());
    }

    #[test]
    fn test_tag_item_item() {
        let i = TagItem::Null.item();
        assert_eq!(i.t_tag, TagItem::Null.symbol());
        assert_eq!(i.t_value, 0);
    }

    #[test]
    fn test_new_tag_item_from_sys() {
        let tag_item = TagItem::NuUrl(CString::new("800@localhost").unwrap());
        let tag_item_sys = tag_item.item();
        let new_tag_item = TagItem::_from_sys(&tag_item_sys);

        assert_eq!(new_tag_item.symbol(), tag_item.symbol());
        assert_eq!(new_tag_item.sofia_string(), tag_item.sofia_string());
    }

    #[test]
    fn test_convert_tag_item_to_tag_and_to_tag_item() {
        let tag_item = TagItem::NuUrl(CString::new("800@localhost").unwrap());
        let tag: Tag = tag_item.into();
        let new_tag_item = TagItem::from(tag);

        assert_eq!(new_tag_item.sofia_string(), "nua::url: <800@localhost>");
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

    #[test]
    fn test_new_tag_type_from_sys() {
        let tt = TagType::_from_sys(unsafe { sys::siptag_content_type_str.as_ptr() });
        assert_eq!(tt.namespace, CString::new("sip").unwrap());
        assert_eq!(tt.name, CString::new("content_type_str").unwrap());
    }
}
