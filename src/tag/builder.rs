use crate::sys;
use crate::tag::Tag;

#[derive(Debug, Clone)]
pub struct Builder {
    tags: Vec<Tag>,
}

pub(crate) fn convert_tags(tags: &Vec<Tag>) -> Vec<sys::tagi_t> {
    let mut sys_tags = Vec::<sys::tagi_t>::new();
    for tag in tags {
        sys_tags.push(tag.item());
    }

    /* last tag must be TAG_END or TAG_NULL */
    let tag_null = Tag::Null;
    sys_tags.push(tag_null.item());
    sys_tags
}

impl Builder {
    pub fn default() -> Self {
        Builder {
            tags: Vec::<Tag>::new(),
        }
    }
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn collect(self) -> Vec<Tag> {
        self.tags
    }
}
