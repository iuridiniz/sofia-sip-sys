// use crate::error::Error;
use crate::result::Result;
use crate::su;
use crate::sys;
use crate::Tag;
use crate::nua::Nua;
use crate::nua::EventClosure;
use crate::nua::Event;

use std::ffi::CStr;
use std::convert::TryFrom;

pub struct Builder {
    root: Option<su::Root>,
    tags: Vec<Tag>,
    closure: Option<Box<EventClosure>>,
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Builder")
            .field("root", &self.root)
            .field("tags", &self.tags)
            .finish()
    }
}

impl Builder {
    pub fn default() -> Self {
        Builder {
            root: None,
            tags: Vec::<Tag>::new(),
            closure: None,
        }
    }
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn callback<F: Fn(&mut Nua, Event, u32, String) + 'static>(mut self, cb: F) -> Self {
        self.closure = Some(Box::new(cb));
        self
    }

    pub fn root(mut self, root: su::Root) -> Self {
        self.root = Some(root);
        self
    }

    pub fn create(self) -> Result<Box<Nua>> {
        let mut nua = Box::new(Nua::_new());
        let nua_ptr = &mut *nua as *mut Nua as *mut sys::nua_magic_t;

        let c_root = match &self.root {
            Some(root) => root.c_ptr,
            _ => crate::su::get_default_root()?.c_ptr,
        };

        let mut sys_tags = Vec::<sys::tagi_t>::new();
        for tag in &self.tags {
            sys_tags.push(tag.item());
        }

        /* last tag must be TAG_END or TAG_NULL */
        let tag_null = Tag::Null;
        sys_tags.push(tag_null.item());

        let sys_tags = sys_tags.as_slice();

        let c_callback = nua_callback_glue;
        let magic = nua_ptr;
        nua.closure = self.closure;
        nua.c_ptr = Nua::_create(c_root, Some(c_callback), magic, Some(sys_tags))?;
        nua.root = self.root;
        Ok(nua)
    }
}


extern "C" fn nua_callback_glue(
    _event: sys::nua_event_t,
    _status: ::std::os::raw::c_int,
    _phrase: *const ::std::os::raw::c_char,
    _nua: *mut sys::nua_t,
    _magic: *mut sys::nua_magic_t,
    _nh: *mut sys::nua_handle_t,
    _hmagic: *mut sys::nua_hmagic_t,
    _sip: *const sys::sip_t,
    _tags: *mut sys::tagi_t,
) {
    dbg!(_event, _status, _phrase, _nua, _magic, _nh, _hmagic, _sip, _tags);

    /*
    Panics can happen pretty much anywhere in Rust code.
    So to be safe, we need to wrap our callback body inside catch_unwind.
    see: https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics
    */
    if let Err(e) = std::panic::catch_unwind(|| {
        /* This call is expect to not panic if sofia does not changes their api */
        /* Also, it can happen if memory is corrupted and the process must be aborted, anyway */
        let event: Event = Event::try_from(_event as i32).unwrap();
        dbg!(&event);
        let status = _status as u32;
        let phrase: String = unsafe { CStr::from_ptr(_phrase).to_string_lossy().into_owned() };
        // let sys_nua: _nua;
        let nua: *mut Nua = _magic as *mut Nua;

        unsafe {
            (*nua).on_sys_nua_event(event, status, phrase, nua);
        }
    }) {
        // Code here must be panic-free.
        eprintln!("PANIC!! while calling a callback from C: {:?}", e);
        // Abort is safe because it doesn't unwind.
        std::process::abort();
    }
}
