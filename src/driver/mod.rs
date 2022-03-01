use core::mem::replace;

use crate::driver::cmos::CMOS;

pub mod cmos;
pub mod ide;
pub mod pci;

static mut PERIPHERALS: Peripherals = Peripherals {
    cmos: Some(CMOS::new()),
};

pub struct Peripherals {
    cmos: Option<CMOS>,
}

impl Peripherals {
    pub fn take_cmos() -> CMOS {
        unsafe {
            let cmos = replace(&mut PERIPHERALS.cmos, None);
            cmos.unwrap()
        }
    }
}
