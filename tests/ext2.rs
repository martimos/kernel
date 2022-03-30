#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::driver::ide::drive::IDEDrive;
use martim::driver::ide::IDEController;
use martim::driver::pci;
use martim::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::io::fs::ext2::Ext2Fs;
use martim::io::fs::Fs;
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
fn test_create_fs() {
    let drive = get_drive();
    let fs = Ext2Fs::new(drive).unwrap();
    let root = fs.root_inode();
    let root_dir = root.dir().unwrap();
    let hello = root_dir
        .read()
        .lookup(&"hello.txt")
        .expect("root dir must have inode 'hello.txt'");
    let hello_file = hello.file().expect("'hello.txt' should be a file");
    let hello_data = hello_file.read().read_full().unwrap();
    let hello_content = String::from_utf8(hello_data).unwrap();
    assert_eq!("hello world", hello_content);
}

fn get_drive() -> IDEDrive {
    let ide_controller = pci::devices()
        .iter()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .cloned()
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    ide_controller
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .nth(1)
        .expect("require one additional drive")
}
