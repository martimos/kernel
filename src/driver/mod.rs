use alloc::rc::Rc;
use core::mem::MaybeUninit;

use spin::{Mutex, Once};

use crate::driver::cmos::{CMOSTime, CMOS};

pub mod cmos;
pub mod ide;
pub mod pci;

pub struct Peripherals;

impl Peripherals {
    pub fn boot_time() -> &'static CMOSTime {
        // TODO: actively call this early on
        static mut BOOT_TIME: MaybeUninit<CMOSTime> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            BOOT_TIME
                .as_mut_ptr()
                .write(Peripherals::cmos().lock().read_time());
        });

        unsafe { &*BOOT_TIME.as_ptr() }
    }

    pub fn cmos() -> Rc<Mutex<CMOS>> {
        static mut CMOS_UNINIT: MaybeUninit<Rc<Mutex<CMOS>>> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            CMOS_UNINIT
                .as_mut_ptr()
                .write(Rc::new(Mutex::new(CMOS::new())));
        });
        unsafe { (*CMOS_UNINIT.as_ptr()).clone() }
    }
}
