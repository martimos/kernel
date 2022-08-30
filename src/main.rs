#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use goblin::elf::header::ET_EXEC;
use goblin::elf::Elf;
use x86_64::instructions::hlt;

use martim::driver::Peripherals;
use martim::io::fs::vfs;
use martim::memory::kbuffer::KBuffer;
use martim::scheduler::Scheduler;
use martim::vfs_setup::init_vfs;
use martim::{debug, hlt_loop, info};
use martim::{
    scheduler, serial_print, serial_println,
    task::{executor::Executor, keyboard, Task},
    vga_clear, vga_println,
};

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    martim::info!(
        "terminating task {}: {}",
        Scheduler::get_current_tid(),
        info
    );

    if Scheduler::get_current_tid().as_usize() == 0 {
        serial_println!("kernel task panicked, halting...");
        hlt_loop()
    } else {
        Scheduler::exit()
    }
}

#[cfg(test)]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("init kernel...");
    martim::init();
    martim::memory::init_memory(boot_info);
    scheduler::init();
    let _ = Peripherals::boot_time(); // initialize boot time
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
    let boot_time = Peripherals::boot_time();
    info!(
        "boot time: {:02}{:02}-{:02}-{:02}, {:02}:{:02}:{:02}",
        boot_time.century.unwrap(),
        boot_time.year,
        boot_time.month,
        boot_time.day_of_month,
        boot_time.hours,
        boot_time.minutes,
        boot_time.seconds,
    );

    Scheduler::spawn_from_c_fn(init_vfs).unwrap();
    Scheduler::spawn_from_c_fn(just_panic).unwrap();
    Scheduler::spawn_from_c_fn(elf_stuff).unwrap();
    Scheduler::spawn_from_c_fn(example_tasks).unwrap();

    debug!(
        "kernel task with tid {} is still running",
        Scheduler::get_current_tid()
    );
}

extern "C" fn elf_stuff() {
    let path = "/mnt/block_device0/executables/hello_world";
    // wait for vfs to be initialized
    let content = loop {
        if let Ok(content) = vfs::read_file_node(&path) {
            break content;
        }
        hlt();
    };
    let elf = Elf::parse(&content).unwrap();
    exec_elf(elf).unwrap();
    info!("executed elf");
}

fn exec_elf(elf: Elf) -> Result<(), ()> {
    if !elf.is_64 {
        return Err(());
    }
    if elf.header.e_type != ET_EXEC {
        return Err(());
    }
    let base_addr = elf
        .program_headers
        .iter()
        .map(|ph| ph.p_vaddr)
        .min()
        .ok_or(())?;
    debug!("base addr: {:#x}", base_addr);
    let required_mem = elf
        .program_headers
        .iter()
        .map(|ph| ph.p_vaddr - base_addr + ph.p_memsz)
        .max()
        .ok_or(())?;
    debug!("required mem: {:#x}", required_mem);

    let elf_buffer = KBuffer::allocate(required_mem as usize);
    debug!("allocated buffer at {:p}", elf_buffer.as_ptr::<usize>());
    Ok(())
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    serial_println!("async number: {}", number);
}

extern "C" fn just_panic() {
    serial_println!("Hi, my name is Tid {} and", Scheduler::get_current_tid());
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
