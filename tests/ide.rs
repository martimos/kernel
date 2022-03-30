#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::driver::ide::IDEController;
use martim::driver::pci::device::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::driver::pci::PCI;
use martim::io::ReadAt;
use martim::scheduler;

entry_point!(main);

#[allow(clippy::empty_loop)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    martim::memory::init_heap(boot_info);
    scheduler::init();
    scheduler::reschedule(); // start the scheduler

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info);
}

#[test_case]
fn test_find_drives() {
    let ide_controller = PCI::devices()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    let drives = ide_controller
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .count();

    assert_eq!(2, drives); // (1) boot drive, (2) disk.img
}

#[test_case]
fn test_read_first_block() {
    let ide_controller = PCI::devices()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    let drive = ide_controller
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .nth(1)
        .expect("require one additional drive");

    let mut block = [0_u8; 512];
    let read_count = drive.read_at(0, &mut block).unwrap();
    assert_eq!(
        block.len(),
        read_count,
        "must have read into the whole input buffer"
    );

    let expected = "Hello, World!";
    let data = String::from_utf8(Vec::from(&block[0..expected.len()])).unwrap();
    assert_eq!(expected, data);
    block
        .iter()
        .skip(expected.len())
        .for_each(|&v| assert_eq!(0, v));
}

#[test_case]
fn test_read_first_block_offset() {
    let ide_controller = PCI::devices()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    let drive = ide_controller
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .nth(1)
        .expect("require one additional drive");

    let mut block = [0_u8; 12];
    let read_count = drive.read_at(1, &mut block).unwrap();
    assert_eq!(block.len(), read_count);

    let expected = "ello, World!"; // we read from offset 1, so the 'H' is truncated
    let data = String::from_utf8(Vec::from(&block[0..expected.len()])).unwrap();
    assert_eq!(expected, data);
}
