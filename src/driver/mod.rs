use kstd::sync::Mutex;

use alloc::sync::Arc;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;

use crate::driver::cmos::{CMOSTime, CMOS};
use crate::driver::ide::drive::IDEDrive;
use crate::driver::ide::IDEController;
use crate::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use crate::driver::pci::header::PCIStandardHeaderDevice;

pub mod cmos;
pub mod ide;
pub mod pci;

pub struct Peripherals;

impl Peripherals {
    pub fn boot_time() -> &'static CMOSTime {
        static BOOT_TIME: OnceCell<CMOSTime> = OnceCell::uninit();

        BOOT_TIME.get_or_init(|| Peripherals::cmos().lock().read_time())
    }

    pub fn cmos() -> &'static Arc<Mutex<CMOS>> {
        static CMOS: OnceCell<Arc<Mutex<CMOS>>> = OnceCell::uninit();
        CMOS.get_or_init(|| Arc::new(Mutex::new(CMOS::new())))
    }

    pub fn ide_controllers() -> &'static [IDEController] {
        static IDE_CONTROLLERS: OnceCell<Vec<IDEController>> = OnceCell::uninit();
        IDE_CONTROLLERS.get_or_init(|| {
            pci::devices()
                .filter(|dev| {
                    dev.class()
                        == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
                })
                .map(|d| PCIStandardHeaderDevice::new(d.clone()).unwrap())
                .map(Into::<IDEController>::into)
                .collect::<Vec<IDEController>>()
        })
    }

    pub fn ide_drives() -> &'static [&'static IDEDrive] {
        static IDE_DRIVES: OnceCell<Vec<&'static IDEDrive>> = OnceCell::uninit();

        IDE_DRIVES.get_or_init(|| {
            Peripherals::ide_controllers()
                .iter()
                .flat_map(|c| c.drives())
                .filter(|d| d.exists())
                .collect::<Vec<&IDEDrive>>()
        })
    }
}
