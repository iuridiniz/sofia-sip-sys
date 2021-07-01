// use crate::error::Error;
use crate::nua::event::Event;
use crate::nua::event::EventClosure;
use crate::nua::handle::Handle;
use crate::nua::Nua;
use crate::result::Result;
use crate::su;
use crate::sys;
use crate::tag::Tag;

use std::convert::TryFrom;
use std::ffi::CStr;

pub struct Builder<'a> {
    root: Option<&'a su::Root>,
    // nua: Option<&'a Nua<'a>>,
    tags: Vec<Tag>,
    closure: Option<Box<EventClosure>>,
}

impl<'a> std::fmt::Debug for Builder<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Builder")
            .field("root", &self.root)
            .field("tags", &self.tags)
            .finish()
    }
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

impl<'a> Builder<'a> {
    pub fn default() -> Self {
        Builder {
            root: None,
            // nua: None,
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

    pub fn root(mut self, root: &'a su::Root) -> Self {
        self.root = Some(root);
        self
    }

    pub fn create_tags(self) -> Vec<Tag> {
        self.tags
    }

    pub fn create_nua(self) -> Result<Box<Nua<'a>>> {
        let mut nua = Box::new(Nua::_new());
        let nua_ptr = &mut *nua as *mut Nua as *mut sys::nua_magic_t;

        let c_root = match &self.root {
            Some(root) => root.c_ptr,
            _ => crate::su::get_default_root()?.c_ptr,
        };

        let tags = convert_tags(&self.tags);
        let sys_tags = tags.as_slice();

        let c_callback = nua_callback_glue;
        let magic = nua_ptr;
        nua.closure = self.closure;
        nua.c_ptr = Nua::_create(c_root, Some(c_callback), magic, Some(sys_tags))?;
        nua.root = self.root;
        Ok(nua)
    }

    pub fn create(self) -> Result<Box<Nua<'a>>> {
        self.create_nua()
    }

    pub fn create_handle(self, nua: &'a Box<Nua<'_>>) -> Result<Box<Handle<'a>>> {
        let mut handle = Box::new(Handle::_new());
        let handle_ptr = &mut *handle as *mut Handle as *mut sys::nua_hmagic_t;

        let tags = convert_tags(&self.tags);
        let sys_tags = tags.as_slice();
        let magic = handle_ptr;

        handle.c_ptr = Handle::_create(nua.c_ptr, magic, Some(sys_tags))?;
        handle.nua = Some(nua);
        Ok(handle)
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
    println!("------ nua_callback_glue ------");

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

        let status = _status as u32;

        let phrase: String = unsafe { CStr::from_ptr(_phrase).to_string_lossy().into_owned() };

        let sys_nua = _nua; /* ignored */

        let nua: *mut Nua = _magic as *mut Nua;

        /* sanity check */
        {
            assert!(!nua.is_null());
            let nua: &Nua = unsafe { &*nua };
            assert_eq!(sys_nua, nua.c_ptr);
        }

        let sys_handle = _nh; /* ignored */

        let handle: *mut Handle = _hmagic as *mut Handle;
        // let handle_struct: Option<&Handle>;
        if !handle.is_null() {
            /* reply to a owned handle function (outgoing sip message) */
            let handle: &Handle = unsafe { &*handle };
            assert_eq!(sys_handle, handle.c_ptr);
            // handle_struct = Some(handle_struct_temp);
        }
        Nua::_on_sys_nua_event(event, status, phrase, nua, handle);
    }) {
        // Code here must be panic-free.
        eprintln!("PANIC!! while calling a callback from C: {:?}", e);
        // Abort is safe because it doesn't unwind.
        std::process::abort();
    }
    println!("------ [nua_callback_glue] ------");
}
