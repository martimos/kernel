#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::driver::ide::IDEController;
use martim::driver::pci::device::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::driver::pci::PCI;
use martim::io::fs::ustar::UstarFs;
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
fn test_read_header_block() {
    let drive = PCI::devices()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .map(PCIStandardHeaderDevice::new)
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work")
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .nth(1)
        .expect("require one additional drive");

    let mut fs = UstarFs::new(drive);
    for path in ["dir/1.txt", "dir/2.txt"] {
        let file = fs
            .open(&path)
            .expect(&*format!("fs must have file '{}'", path));
        let content = String::from_utf8(Vec::from(file.data())).unwrap();
        assert_eq!("hello\n", content);
    }
}
