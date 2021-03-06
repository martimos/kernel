#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::device::block::BlockDevice;
use martim::driver::ide::drive::IDEDrive;
use martim::driver::ide::IDEController;
use martim::driver::pci;
use martim::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
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

fn get_ide_controller() -> IDEController {
    pci::devices()
        .iter()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .cloned()
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work")
}

fn get_ide_drive(drive_num: usize) -> IDEDrive {
    get_ide_controller()
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .nth(drive_num)
        .expect("require one additional drive")
}

#[test_case]
fn test_find_drives() {
    let drives = get_ide_controller()
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .count();

    assert_eq!(2, drives); // (1) boot drive, (2) disk.img
}

#[test_case]
fn test_read_first_block() {
    let drive = get_ide_drive(1);

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
    let drive = get_ide_drive(1);

    let mut block = [0_u8; 12];
    let read_count = drive.read_at(1, &mut block).unwrap();
    assert_eq!(block.len(), read_count);

    let expected = "ello, World!"; // we read from offset 1, so the 'H' is truncated
    let data = String::from_utf8(Vec::from(&block[0..expected.len()])).unwrap();
    assert_eq!(expected, data);
}

#[test_case]
fn test_write_first_block() {
    let mut drive = get_ide_drive(1);

    let original_block = {
        let mut data = vec![0_u8; 512];
        drive.read_block(0, &mut data).unwrap();
        data
    };

    let write_data = {
        let mut data = vec![0_u8; 512];
        data.fill(0xDE);
        data
    };
    drive.write_block(0, &write_data).unwrap();

    let mut read_back = vec![0_u8; 512];
    drive.read_block(0, &mut read_back).unwrap();
    assert_eq!(write_data, read_back);

    // write back original data
    drive.write_block(0, &original_block).unwrap();

    let mut original_read_back = vec![0_u8; 512];
    drive.read_block(0, &mut original_read_back).unwrap();
    assert_eq!(original_block, original_read_back); // if this fails, writing back the original block failed
}
