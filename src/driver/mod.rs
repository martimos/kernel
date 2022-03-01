use alloc::sync::Arc;

use spin::Mutex;

use crate::driver::cmos::CMOS;

pub mod cmos;
pub mod ide;
pub mod pci;

static mut PERIPHERALS: Peripherals = Peripherals { cmos: None };

pub struct Peripherals {
    cmos: Option<Arc<Mutex<CMOS>>>,
}

impl Peripherals {
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

        self.cmos = Some(Arc::new(Mutex::new(CMOS::new())));
    }
}
