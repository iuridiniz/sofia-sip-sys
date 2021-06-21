pub mod error;
pub mod nua;
pub mod result;
pub mod su;
pub mod sys;
pub use nua::Nua;
pub use su::main_loop_run;

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
    fn to_tag_list(&self) -> Vec<sys::tagi_t> {
        vec![]
    }
}
