#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::filesystem::vfs;
#[cfg(not(test))]
use martim::hlt_loop;
use martim::task::executor::Executor;
use martim::task::{keyboard, Task};
use martim::{serial_print, serial_println, vga_clear, vga_println};

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}, halting", info);
    hlt_loop()
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
    martim::init_heap(boot_info);
    martim::multitasking::init();
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

    // for i in 1.. {
    //     vga_println!("{}", i);
    //     hlt();
    // }

    #[cfg(not(test))]
    main();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::default();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(example_task()));
    executor.run();
}

fn main() {
    vga_println!("Hello, {}!", "World");
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    vga_println!("async number: {}", number);
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}
