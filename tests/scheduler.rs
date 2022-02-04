#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicU32, Ordering},
};

use bootloader::{entry_point, BootInfo};
use martim::{scheduler, scheduler::priority::NORMAL_PRIORITY};
use x86_64::instructions::hlt;

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    martim::memory::init_heap(boot_info);
    scheduler::init();

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info)
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[test_case]
fn counter_many_tasks_nohlt() {
    COUNTER.store(0, Ordering::SeqCst);

    // This creates 10 tasks, each with their own stack. Unlikely, but when
    // we allocate more stack per task, this may cause a kernel out of memory.
    // In that case, either reduce this number (should stay >1) or increase
    // the total kernel memory.
    const NUM_TASKS: u32 = 10;

    for _ in 0..NUM_TASKS {
        scheduler::spawn(counter_many_tasks_nohlt_fn, NORMAL_PRIORITY).unwrap();
    }

    scheduler::reschedule();

    assert_eq!(NUM_TASKS * 1000, COUNTER.load(Ordering::SeqCst));
}

extern "C" fn counter_many_tasks_nohlt_fn() {
    for _ in 0..1000 {
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}

#[test_case]
fn counter_two_tasks() {
    COUNTER.store(0, Ordering::SeqCst);

    scheduler::spawn(counter_two_tasks_fn, NORMAL_PRIORITY).unwrap();
    scheduler::spawn(counter_two_tasks_fn, NORMAL_PRIORITY).unwrap();

    scheduler::reschedule();

    assert_eq!(10, COUNTER.load(Ordering::SeqCst));
}

extern "C" fn counter_two_tasks_fn() {
    for _ in 0..5 {
        COUNTER.fetch_add(1, Ordering::SeqCst);
        hlt();
    }
}
