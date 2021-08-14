use crate::sys;
use crate::tag::tag::Tag;
use crate::tag::tag::TagItem;

#[derive(Debug, Clone)]
pub struct Builder {
    tags: Vec<Tag>,
}

pub(crate) fn convert_tags(tags: &[TagItem]) -> Vec<sys::tagi_t> {
    let mut sys_tags = Vec::<sys::tagi_t>::with_capacity(tags.len() + 1);
    for tag in tags {
        sys_tags.push(tag.item());
    }

    /* last tag must be TAG_END or TAG_NULL */
    let tag_null = TagItem::Null;
    sys_tags.push(tag_null.item());
    sys_tags
}

impl Builder {
    pub fn default() -> Self {
        Builder {
            tags: Vec::<Tag>::new(),
        }
    }

    pub fn nutag_url(self, url: &str) -> Self {
        self.tag(Tag::NuUrl(url.to_string()))
    }

    pub fn nutag_m_username(self, s: &str) -> Self {
        self.tag(Tag::NuMUsername(s.to_string()))
    }

    pub fn nutag_m_display(self, s: &str) -> Self {
        self.tag(Tag::NuMDisplay(s.to_string()))
    }

    pub fn soatag_user_sdp_str(self, s: &str) -> Self {
        self.tag(Tag::SoaUserSdpStr(s.to_string()))
    }

    pub fn siptag_subject_str(self, s: &str) -> Self {
        self.tag(Tag::SipSubjectStr(s.to_string()))
    }

    pub fn siptag_content_type_str(self, s: &str) -> Self {
        self.tag(Tag::SipContentTypeStr(s.to_string()))
    }

    pub fn siptag_payload_str(self, s: &str) -> Self {
        self.tag(Tag::SipPayloadStr(s.to_string()))
    }

    pub fn siptag_to_str(self, s: &str) -> Self {
        self.tag(Tag::SipToStr(s.to_string()))
    }

    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn collect(self) -> Vec<Tag> {
        self.tags
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_collect() {
        let res = Builder::default().collect();
        assert_eq!(res.len(), 0);

        let res = Builder::default()
            .tag(Tag::NuUrl("800@localhost".to_string()))
            .collect();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_builder_nutag_url() {
        let res = Builder::default().nutag_url("800@localhost").collect();
        assert_eq!(res[0], Tag::NuUrl("800@localhost".to_string()));
    }

    #[test]
    fn test_builder_nutag_m_username() {
        let res = Builder::default().nutag_m_username("Alice").collect();
        assert_eq!(res[0], Tag::NuMUsername("Alice".to_string()));
    }

    #[test]
    fn test_builder_nutag_m_display() {
        let res = Builder::default().nutag_m_username("Alice").collect();
        assert_eq!(res[0], Tag::NuMUsername("Alice".to_string()));
    }

    #[test]
    fn test_builder_soatag_user_sdp_str() {
        let res = Builder::default().soatag_user_sdp_str("O=A").collect();
        assert_eq!(res[0], Tag::SoaUserSdpStr("O=A".to_string()));
    }

    #[test]
    fn test_builder_siptag_subject_str() {
        let res = Builder::default().siptag_subject_str("Subject").collect();
        assert_eq!(res[0], Tag::SipSubjectStr("Subject".to_string()));
    }

    #[test]
    fn test_builder_siptag_content_type_str() {
        let res = Builder::default()
            .siptag_content_type_str("Content")
            .collect();
        assert_eq!(res[0], Tag::SipContentTypeStr("Content".to_string()));
    }

    #[test]
    fn test_builder_siptag_payload_str() {
        let res = Builder::default().siptag_payload_str("Payload").collect();
        assert_eq!(res[0], Tag::SipPayloadStr("Payload".to_string()));
    }

    #[test]
    fn test_builder_siptag_to_str() {
        let res = Builder::default().siptag_to_str("900@localhost").collect();
        assert_eq!(res[0], Tag::SipToStr("900@localhost".to_string()));
    }
}
