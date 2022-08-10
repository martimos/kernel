#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::driver::ide::IDEController;
use martim::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::driver::{pci, Peripherals};
use martim::io::fs::devfs::DevFs;
use martim::io::fs::memfs::MemFs;
use martim::io::fs::{vfs, Fs};
use martim::{debug, hlt_loop, info};
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
    let _ = Peripherals::boot_time(); // initialize boot time
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

    let boot_time = Peripherals::boot_time();
    info!(
        "Boot time: {:02}{:02}-{:02}-{:02}, {:02}:{:02}:{:02}",
        boot_time.century.unwrap(),
        boot_time.year,
        boot_time.month,
        boot_time.day_of_month,
        boot_time.hours,
        boot_time.minutes,
        boot_time.seconds,
    );

    scheduler::spawn(just_panic).unwrap();
    scheduler::spawn(vfs_setup).unwrap();
    scheduler::spawn(ide_drives).unwrap();
    scheduler::spawn(example_tasks).unwrap();

    debug!(
        "kernel task with tid {} is still running",
        scheduler::get_current_tid()
    );
}

extern "C" fn vfs_setup() {
    let devfs = DevFs::new("dev".to_string());
    vfs::mount(&"/", devfs.root_inode()).unwrap();

    let memfs = MemFs::new("mem".to_string());
    vfs::mount(&"/dev", memfs.root_inode()).unwrap();

    vfs::find_inode(&"/dev/serial")
        .expect("/dev/serial not found")
        .file()
        .expect("/dev/serial is not a file")
        .write()
        .write_at(
            0,
            b"This was written to serial port via the file /dev/serial!\n",
        )
        .expect("failed to write to /dev/serial");
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
