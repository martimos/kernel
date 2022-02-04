#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    use martim::{
        allocator,
        memory::{self, BootInfoFrameAllocator},
    };
    use x86_64::VirtAddr;

    martim::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info)
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, collections::LinkedList, vec, vec::Vec};
    use martim::allocator::HEAP_SIZE;

    #[test_case]
    fn simple_allocation_box_syntax() {
        let heap_value_1 = box 41;
        let heap_value_2 = box 13;
        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
    }
    #[test_case]
    fn simple_allocation() {
        let heap_value_1 = Box::new(41);
        let heap_value_2 = Box::new(13);
        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
    }

    #[test_case]
    fn simple_linked_list() {
        let mut l = Box::new(LinkedList::<&str>::new());
        l.push_back("hello");
        assert_eq!(1, l.len());
    }

    #[test_case]
    fn large_vec() {
        let n = 1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    }

    /// All these allocations can't fit into heap, so this tests
    /// whether the allocator can re-use previous, freed allocations
    #[test_case]
    fn many_boxes() {
        for i in 0..HEAP_SIZE {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
    }

    #[test_case]
    fn many_boxes_long_lived() {
        let long_lived = Box::new(1);
        for i in 0..HEAP_SIZE {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
        assert_eq!(*long_lived, 1);
    }

    #[test_case]
    fn many_boxes_long_lived_box_syntax() {
        let long_lived = box 1;
        for i in 0..HEAP_SIZE {
            let x = box i;
            assert_eq!(*x, i);
        }
        assert_eq!(*long_lived, 1);
    }

    #[test_case]
    fn large_allocation() {
        let size: usize = HEAP_SIZE / 2;
        for _ in 0..1000 {
            let large = vec![0_u8; size];
            assert_eq!(large.len(), size);
        }
    }

    #[test_case]
    fn large_allocation_box_syntax() {
        const SIZE: usize = HEAP_SIZE / 2;
        for _ in 0..1000 {
            let large = box [0_u8; SIZE];
            assert_eq!(large.len(), SIZE);
        }
    }
}
