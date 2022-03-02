#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::AtomicBool;
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicU32, Ordering},
};

use bootloader::{entry_point, BootInfo};
use x86_64::instructions::hlt;

use martim::scheduler;

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    martim::memory::init_heap(boot_info);
    scheduler::init();
    scheduler::reschedule(); // start the scheduler

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info);
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

    scheduler::reschedule();

    let mut tids = Vec::new();
    for _ in 0..NUM_TASKS {
        tids.push(scheduler::spawn(counter_many_tasks_nohlt_fn).unwrap());
    }
    for tid in tids {
        scheduler::join(tid);
    }

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

    let tid1 = scheduler::spawn(counter_two_tasks_fn).unwrap();
    let tid2 = scheduler::spawn(counter_two_tasks_fn).unwrap();

    scheduler::reschedule();

    scheduler::join(tid1);
    scheduler::join(tid2);

    assert_eq!(10, COUNTER.load(Ordering::SeqCst));
}

extern "C" fn counter_two_tasks_fn() {
    for _ in 0..5 {
        COUNTER.fetch_add(1, Ordering::SeqCst);
        hlt();
    }
}

static FINISHED: AtomicBool = AtomicBool::new(false);

/// If stacks don't get deallocated, we will run into an allocation failure
/// way before we've reached 10000 tasks (at least with the current stack model).
#[test_case]
fn task_stack_is_deallocated() {
    scheduler::spawn(task_stack_is_deallocated_spawner).unwrap();

    scheduler::reschedule();
}

const MAX_CONCURRENT: u32 = 10;

extern "C" fn task_stack_is_deallocated_spawner() {
    COUNTER.store(0, Ordering::SeqCst);
    FINISHED.store(false, Ordering::SeqCst);

    for _ in 0..10000 {
        let i = COUNTER.load(Ordering::SeqCst);
        if i > MAX_CONCURRENT {
            scheduler::reschedule();
        } else {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            scheduler::spawn(task_stack_is_deallocated_fn).unwrap();
        }
    }
    FINISHED.store(true, Ordering::SeqCst);
}

extern "C" fn task_stack_is_deallocated_fn() {
    while COUNTER.load(Ordering::SeqCst) < MAX_CONCURRENT && !FINISHED.load(Ordering::SeqCst) {
        scheduler::reschedule();
    }
    COUNTER.fetch_sub(1, Ordering::SeqCst);
}

#[test_case]
fn test_no_deadlock_when_join_self() {
    let current_tid = scheduler::get_current_tid();
    scheduler::join(current_tid); // this must return immediately
}
