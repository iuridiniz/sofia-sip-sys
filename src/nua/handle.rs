use crate::error::Error;
use crate::nua::Nua;
use crate::result::Result;
use crate::sys;
use crate::tag::builder::convert_tags;
use crate::tag::Tag;

#[derive(Debug)]
pub struct Handle<'a> {
    pub(crate) nua: Option<&'a Nua<'a>>,
    pub(crate) c_ptr: *mut sys::nua_handle_t,
}

impl<'a> Handle<'a> {
    pub(crate) fn _new() -> Handle<'a> {
        Handle {
            nua: None,
            c_ptr: std::ptr::null_mut(),
        }
    }

    pub fn create(nua: &'a Box<Nua<'_>>, tags: &[Tag]) -> Result<Box<Handle<'a>>> {
        // todo!()
        let mut handle = Box::new(Self::_new());
        let handle_ptr = &mut *handle as *mut Handle as *mut sys::nua_hmagic_t;

        let tags = convert_tags(&tags);
        let sys_tags = tags.as_slice();
        let magic = handle_ptr;
        handle.c_ptr = Handle::_create(nua.c_ptr, magic, Some(sys_tags))?;
        handle.nua = Some(nua);
        Ok(handle)
    }

    pub(crate) fn _create(
        nua: *mut sys::nua_t,
        magic: *mut sys::nua_hmagic_t,
        tags: Option<&[sys::tagi_t]>,
    ) -> Result<*mut sys::nua_handle_t> {
        let tag_name: *const sys::tag_type_s;
        let tag_value: isize;

        if nua.is_null() {
            return Err(Error::CreateNuaHandleError);
        }

        if magic.is_null() {
            return Err(Error::CreateNuaHandleError);
        }

        if tags.is_none() {
            /* TAG_NULL */
            tag_name = std::ptr::null();
            tag_value = 0;
        } else {
            /* TAG_NEXT */
            tag_name = unsafe { sys::tag_next.as_ptr() };
            tag_value = tags.unwrap().as_ptr() as isize;
        }

        let handle_sys = unsafe { sys::nua_handle(nua, magic, tag_name, tag_value) };
        if handle_sys.is_null() {
            /* failed to create */
            return Err(Error::CreateNuaHandleError);
        }
        Ok(handle_sys)
    }

    pub(crate) fn _message(nh: *mut sys::nua_handle_t, tags: Option<&[sys::tagi_t]>) {
        let tag_name: *const sys::tag_type_s;
        let tag_value: isize;

        assert!(!nh.is_null());

        // dbg!(tags);

        if tags.is_none() {
            /* TAG_NULL */
            tag_name = std::ptr::null();
            tag_value = 0;
        } else {
            /* TAG_NEXT */
            tag_name = unsafe { sys::tag_next.as_ptr() };
            tag_value = tags.unwrap().as_ptr() as isize;
        }
        unsafe { sys::nua_message(nh, tag_name, tag_value) };
    }

    pub fn message(&self, tags: &[Tag]) {
        let tags = convert_tags(tags);
        let sys_tags = tags.as_slice();

        let nh = self.c_ptr;
        Self::_message(nh, Some(sys_tags))
    }

    /* FIXME: missing call to destroy (memory leak will occurs) */
}
