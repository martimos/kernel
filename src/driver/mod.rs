use alloc::sync::Arc;

use spin::Mutex;

use crate::driver::cmos::{CMOSTime, CMOS};

pub mod cmos;
pub mod ide;
pub mod pci;

static mut PERIPHERALS: Peripherals = Peripherals {
    cmos: None,
    boot_time: None,
};

pub struct Peripherals {
    cmos: Option<Arc<Mutex<CMOS>>>,
    boot_time: Option<CMOSTime>,
}

impl Peripherals {
    pub fn boot_time(&self) -> CMOSTime {
        self.boot_time.expect("cmos not initialized yet")
    }

    pub fn cmos() -> Arc<Mutex<CMOS>> {
        unsafe {
            PERIPHERALS.init_cmos();
            PERIPHERALS.cmos.as_ref().unwrap().clone()
        }
    }

    fn init_cmos(&mut self) {
        if self.cmos.is_some() {
            return;
        }

        let mut cmos = CMOS::new();
        self.boot_time = Some(cmos.read_time());
        self.cmos = Some(Arc::new(Mutex::new(cmos)));
    }
}
