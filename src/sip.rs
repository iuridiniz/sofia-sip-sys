use crate::sys;

#[derive(Default, Debug)]
struct SipAddr {}

#[derive(Default, Debug)]
pub struct Sip {
    from: SipAddr,
}
impl Sip {
    pub(crate) fn _from_sip_t(sys_sip: *const sys::sip_t) -> Self {
        let sip = Sip::default();
        if sys_sip.is_null() {
            return sip;
        }

        sip
    }
}
