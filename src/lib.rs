pub mod error;
pub mod nua;
pub mod result;
pub mod su;
pub mod sys;
pub mod tag;
pub use nua::Event as NuaEvent;
pub use nua::Nua;
pub use nua::Builder as NuaBuilder;
pub use su::get_default_root;
pub use tag::Tag;

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
