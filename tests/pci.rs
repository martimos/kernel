#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::driver::pci::device::{
    BridgeSubClass, MassStorageSubClass, NetworkSubClass, PCIDeviceClass,
};

use martim::driver::pci::PCI;

use martim::scheduler;

entry_point!(main);

#[allow(clippy::empty_loop)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    martim::memory::init_heap(boot_info);
    scheduler::init();

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info);
}

#[test_case]
fn test_pci_devices_exist() {
    use BridgeSubClass::*;
    use MassStorageSubClass::*;
    use NetworkSubClass::*;
    use PCIDeviceClass::*;

    for (count, class) in [
        (1, Bridge(HostBridge)),                    // always should be present
        (1, NetworkController(EthernetController)), // qemu mounts this by default
        (1, MassStorageController(IDEController)),  // for the boot drive
                                                    // no display device since tests are booted with '-display none'
    ] {
        assert_eq!(count, PCI::devices().filter(|d| d.class() == class).count());
    }
}
