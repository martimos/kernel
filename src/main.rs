#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use core::arch::asm;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use x86_64::registers::segmentation::{Segment, DS, ES, FS, GS};
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;

use martim::driver::ide::IDEController;
use martim::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::driver::{pci, Peripherals};
use martim::io::fs::devfs::DevFs;
use martim::io::fs::memfs::MemFs;
use martim::io::fs::{vfs, Fs};
use martim::{debug, hlt_loop};
use martim::{
    scheduler, serial_print, serial_println,
    task::{executor::Executor, keyboard, Task},
    vga_clear, vga_println,
};

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::info!(
        "terminating task {}: {}",
        scheduler::get_current_tid(),
        info
    );

    if scheduler::get_current_tid().as_usize() == 0 {
        serial_println!("kernel task panicked, halting...");
        hlt_loop()
    } else {
        scheduler::exit()
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("init kernel...");
    martim::init();
    martim::memory::init_heap(boot_info);
    scheduler::init();
    vfs::init();
    serial_println!("done");

    vga_clear!();
    vga_println!(
        r#"

$$\      $$\                      $$\     $$\
$$$\    $$$ |                     $$ |    \__|
$$$$\  $$$$ | $$$$$$\   $$$$$$\ $$$$$$\   $$\ $$$$$$\$$$$\
$$\$$\$$ $$ | \____$$\ $$  __$$\\_$$  _|  $$ |$$  _$$  _$$\
$$ \$$$  $$ | $$$$$$$ |$$ |  \__| $$ |    $$ |$$ / $$ / $$ |
$$ |\$  /$$ |$$  __$$ |$$ |       $$ |$$\ $$ |$$ | $$ | $$ |
$$ | \_/ $$ |\$$$$$$$ |$$ |       \$$$$  |$$ |$$ | $$ | $$ |
\__|     \__| \_______|\__|        \____/ \__|\__| \__| \__|

"#
    );

    #[cfg(not(test))]
    main();

    #[cfg(test)]
    test_main();

    hlt_loop()
}

fn main() {
    vga_println!("Hello, {}!", "World");

    // scheduler::spawn(just_panic).unwrap();
    // scheduler::spawn(vfs_setup).unwrap();
    // scheduler::spawn(cmos_stuff).unwrap();
    // scheduler::spawn(ide_drives).unwrap();
    // scheduler::spawn(example_tasks).unwrap();
    //
    // debug!(
    //     "kernel task with tid {} is still running",
    //     scheduler::get_current_tid()
    // );

    usermode_stuff();
}

fn usermode_stuff() {
    debug!("entering usermode");
    let entry = usermode_entry as *const () as u64;
    debug!("entry pointer: {:#X?}", entry);
    unsafe {
        let data_selector = SegmentSelector::new(4, PrivilegeLevel::Ring3);

        let code_selector = SegmentSelector::new(3, PrivilegeLevel::Ring3);

        let rsp: u64;
        asm!("mov {rsp}, rsp", rsp = out(reg) rsp);
        serial_println!("rsp: {:#X?}", rsp);

        DS::set_reg(data_selector);
        ES::set_reg(data_selector);
        FS::set_reg(data_selector);
        GS::set_reg(data_selector);
        // prepare the stack frame for iret
        asm!(
            "push {data_selector}", // user data
            "push {rsp}", // stack pointer
            "pushfq", // eflags
            "push {code_selector}", // user code
            "push {entry}", // entry point / rip
            "iretq",
            rsp = in(reg) rsp,
            data_selector = in(reg) data_selector.0,
            code_selector = in(reg) code_selector.0,
            entry = in(reg) entry,
        );
    }
}

fn usermode_entry() {
    unsafe {
        asm!("cli", options(nomem, noreturn)); // hopefully triggers a general protection fault
    }
}

extern "C" fn vfs_setup() {
    let memfs = MemFs::new("mem".to_string());
    vfs::mount(&"/", memfs.root_inode()).unwrap();
    let devfs = DevFs::new("dev".to_string());
    vfs::mount(&"/", devfs.root_inode()).unwrap();
}

extern "C" fn cmos_stuff() {
    let cmos = Peripherals::cmos();
    let mut guard = cmos.lock();
    let time = guard.read_time();
    vga_println!(
        "CMOS time: {:02}{:02}-{:02}-{:02}, {:02}:{:02}:{:02}",
        time.century.unwrap(),
        time.year,
        time.month,
        time.day_of_month,
        time.hours,
        time.minutes,
        time.seconds
    );
}

extern "C" fn ide_drives() {
    let ide_controller = pci::devices()
        .iter()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .cloned()
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    for drive in ide_controller.drives().iter().filter(|d| d.exists()) {
        vga_println!(
            "found IDE drive at ctrlbase={:#X} iobase={:#X} drive={:#X}",
            drive.ctrlbase(),
            drive.iobase(),
            drive.drive_num()
        );
    }
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    vga_println!("async number: {}", number);
}

extern "C" fn just_panic() {
    serial_println!("Hi, my name is Tid {} and", scheduler::get_current_tid());
    panic!("Welcome to MartimOS");
}

extern "C" fn example_tasks() {
    let mut executor = Executor::default();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

#[cfg(test)]
mod tests {
    use core::assert_eq;

    #[allow(clippy::eq_op)]
    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}
