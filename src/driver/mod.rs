use kstd::sync::{Mutex, Once};

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::mem::MaybeUninit;

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

    pub fn ide_controllers() -> &'static [IDEController] {
        static mut IDE_CONTROLLERS: MaybeUninit<Vec<IDEController>> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            let ide_controllers = pci::devices()
                .iter()
                .filter(|dev| {
                    dev.class()
                        == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
                })
                .map(|d| PCIStandardHeaderDevice::new(d.clone()).unwrap())
                .map(Into::<IDEController>::into)
                .collect::<Vec<IDEController>>();
            IDE_CONTROLLERS.as_mut_ptr().write(ide_controllers);
        });

        unsafe { &*IDE_CONTROLLERS.as_ptr() }
    }

    pub fn ide_drives() -> &'static [&'static IDEDrive] {
        static mut IDE_DRIVES: MaybeUninit<Vec<&'static IDEDrive>> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| unsafe {
            let ide_drives = Peripherals::ide_controllers()
                .iter()
                .flat_map(|c| c.drives())
                .filter(|d| d.exists())
                .collect::<Vec<&IDEDrive>>();
            IDE_DRIVES.as_mut_ptr().write(ide_drives);
        });

        unsafe { &*IDE_DRIVES.as_ptr() }
    }
}
