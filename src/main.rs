#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::driver::ide::IDEController;
use martim::driver::pci::device::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::driver::Peripherals;
use martim::filesystem::vfs::Vfs;
#[cfg(not(test))]
use martim::hlt_loop;
use martim::{
    driver::pci::PCI,
    info, scheduler,
    scheduler::priority::NORMAL_PRIORITY,
    serial_print, serial_println,
    task::{executor::Executor, keyboard, Task},
    vga_clear, vga_println,
};

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    info!(
        "terminating task {}: {}",
        scheduler::get_current_pid(),
        info
    );

    if scheduler::get_current_pid().as_usize() == 0 {
        serial_println!("kernel task panicked, halting...");
        hlt_loop()
    } else {
        scheduler::do_exit()
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

    panic!("main returned")
}

fn main() {
    vga_println!("Hello, {}!", "World");

    let vfs = Vfs::new();
    let path = "/dev/zero";
    let res = vfs.find_vnode(&path);
    info!("open {}: {:?}", path, res);

    scheduler::spawn(just_panic, NORMAL_PRIORITY).unwrap();
    scheduler::spawn(cmos_stuff, NORMAL_PRIORITY).unwrap();
    // scheduler::spawn(pci_stuff, NORMAL_PRIORITY).unwrap();
    scheduler::spawn(example_tasks, NORMAL_PRIORITY).unwrap();

    info!("starting scheduler");
    scheduler::reschedule();
}

extern "C" fn cmos_stuff() {
    let mut cmos = Peripherals::take_cmos();
    let time = cmos.read_time();
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

extern "C" fn pci_stuff() {
    for dev in PCI::devices() {
        serial_println!(
            "pci device on bus {}, slot {}, function {}: {:X}:{:X}\n\theader type: {:?} (mf: {})\n\tclass/prog: {:?}/{:#X}\n\tstatus: {:?}",
            dev.bus(),
            dev.slot(),
            dev.function(),
            dev.vendor(),
            dev.device(),
            dev.header_type(),
            dev.is_multi_function(),
            dev.class(),
            dev.prog_if(),
            dev.status(),
        );
    }

    let ide_controller = PCI::devices()
        .filter(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .next()
        .map(PCIStandardHeaderDevice::new)
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    info!("listing ATA drives:");
    for drive in ide_controller.drives().iter().filter(|d| d.exists()) {
        serial_println!("{:#?}", drive);
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
    serial_println!("Hi, my name is Pid {} and", scheduler::get_current_pid());
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

    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}
