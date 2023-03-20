#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use martim::memory;

entry_point!(main);

#[allow(clippy::empty_loop)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    memory::init_memory(boot_info);

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info)
}

#[cfg(test)]
mod tests {
    use martim::memory::kbuffer::KBuffer;

    #[test_case]
    fn test_kbuffer_allocation() {
        let buf = KBuffer::allocate(100);
        assert_eq!(100, buf.len());
        assert_eq!(
            0,
            buf.as_ptr::<u8>() as usize % 64,
            "kbuffers are always aligned to 64 bytes"
        );
    }

    #[test_case]
    fn test_asref_u8_slice() {
        let buf = KBuffer::allocate(100);
        let slice = buf.as_ref();
        assert_eq!(100, slice.len());
        slice.iter().for_each(|&x| {
            assert_eq!(0, x);
        });
    }

    #[test_case]
    fn test_asmut_u8_slice() {
        let mut buf = KBuffer::allocate(100);
        let slice = buf.as_mut();
        assert_eq!(100, slice.len());
        slice.iter_mut().enumerate().for_each(|(i, x)| {
            *x = (i % 255) as u8;
        });
        slice.iter().enumerate().for_each(|(i, x)| {
            assert_eq!((i % 255) as u8, *x);
        });
    }
}
