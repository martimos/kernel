#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::io::ReadAt;
use martim::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    martim::memory::init_heap(boot_info);

    should_fail();

    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    let data = vec![0_u8, 1];
    let mut buf = vec![0_u8; 5];

    serial_print!("read_at_out_of_bounds_should_panic::should_fail...\t");
    data.read_at(4, &mut buf); // don't unwrap, we want to make sure that the read_at itself panics
}
