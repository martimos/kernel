#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::{serial_print, serial_println};

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("init kernel...");
    martim::init();
    martim::init_heap(boot_info);
    serial_println!("done");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info);
}

#[cfg(test)]
mod tests {
    use martim::{vga_clear, vga_print, vga_println};

    #[test_case]
    fn test_vga_println_no_panic() {
        for _ in 1..100 {
            vga_println!("this must not panic");
        }
    }

    #[test_case]
    fn test_vga_clear_no_panic() {
        for _ in 1..2 {
            vga_println!("this must not panic");
        }

        vga_clear!();

        for _ in 1..2 {
            vga_println!("this must not panic");
        }
    }

    #[test_case]
    fn test_vga_full_buffer_scroll() {
        // try to write 4MiB of 'c's without explicit new lines
        for _ in 1..(1 << 12) {
            vga_print!("c");
        }
    }
}
