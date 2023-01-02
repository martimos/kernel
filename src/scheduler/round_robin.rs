use alloc::collections::VecDeque;
use core::mem::swap;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use x86_64::instructions::interrupts::without_interrupts;

use crate::scheduler::switch::switch;
use crate::scheduler::task::{ProcessStatus, Task};
use crate::scheduler::tid::Tid;
use crate::syscall::Result;
use crate::{debug, hlt_loop};
use kstd::collections::deltaq::DeltaQueue;

pub struct RoundRobin {
    current_task: Task,
    /// Tasks that are ready to be scheduled.
    ready_queue: VecDeque<Task>,
    /// Finished tasks waiting for cleanup.
    finished_tasks: VecDeque<Task>,
    sleeping_tasks: DeltaQueue<Task>,
    /// The amount of running or ready tasks.
    /// Finished tasks are not included.
    task_count: AtomicU32,
    ticks: u64,
}

impl !Default for RoundRobin {}

impl RoundRobin {
    pub fn new() -> Self {
        let current_task = Task::new_for_current(Tid::new());

        Self {
            current_task,
            ready_queue: VecDeque::new(),
            finished_tasks: VecDeque::new(),
            sleeping_tasks: DeltaQueue::new(),
            task_count: AtomicU32::new(1), // 1 since the current_task already exists
            ticks: 0,
        }
    }

    pub fn spawn(&mut self, func: NonNull<usize>) -> Result<Tid> {
        without_interrupts(|| {
            // Create the new task.
            let tid = Tid::new();
            let mut task = Task::new(tid, ProcessStatus::Ready);

            task.allocate_stack(func);

            // Add it to the task lists.
            self.ready_queue.push_back(task);
            self.task_count.fetch_add(1, Ordering::SeqCst);

            Ok(tid)
        })
    }

    pub fn cpu_time(&mut self) -> Duration {
        // TODO: currently, it feels like the interrupts occur in 100ms intervals, so use that, but it's probably inaccurate
        Duration::from_millis(self.current_task.ticks * 100)
    }

    /// Terminates the currently running task and reschedules,
    /// so that the next available task will be run.
    pub fn exit(&mut self) -> ! {
        self.current_task.status = ProcessStatus::Finished;
        self.task_count.fetch_sub(1, Ordering::SeqCst);

        hlt_loop() // just hlt until this is finally collected
    }

    pub fn sleep(&mut self, duration: Duration) {
        without_interrupts(|| {
            let current_task = &mut self.current_task;
            debug!(
                "put task {} to sleep for {}ms",
                current_task.tid,
                duration.as_millis()
            );
            current_task.sleep_ticks = (duration.as_millis() / 50) as usize; // TODO: divide by timer tick duration, 5 feels right in the test that was used while implementing this
            current_task.status = ProcessStatus::Sleeping;
        });

        self.reschedule()
    }

    /// Returns the task id (tid) of the currently running task.
    pub fn get_current_tid(&self) -> Tid {
        self.current_task.tid
    }

    pub fn total_ticks(&self) -> u64 {
        self.ticks
    }

    pub fn timer_tick(&mut self) {
        self.ticks += 1
    }

    pub fn reschedule(&mut self) {
        // TODO: If there are finished tasks waiting for deallocation, we should do that first.
        // One task cleanup per schedule should on average be enough (hopefully)
        // to not accumulate a whole pile of finished, not cleaned up, tasks.

        let mut switch_args: Option<(*mut usize, *const usize)> = None;

        without_interrupts(|| {
            // TODO: unwind and properly deallocate tasks from the finished queue

            // TODO: create tests for this
            let maybe_next_task = {
                let sleeping_task_ready = match self.sleeping_tasks.front() {
                    Some(n) => n.value == 0,
                    None => false,
                };
                if sleeping_task_ready {
                    self.sleeping_tasks.pop_front()
                } else {
                    self.ready_queue.pop_front()
                }
            };
            {
                // decrement ticks in sleeping queue
                match self.sleeping_tasks.front_mut() {
                    None => {}
                    Some(n) => {
                        // Only decrement if the front value is not already zero.
                        // If it is, then the task is already ready to be scheduled.
                        // This shifts the whole queue back by one tick.
                        if n.value > 0 {
                            n.value -= 1
                        }
                    }
                };
            }

            if maybe_next_task.is_none() {
                // this is basically the idle implementation - do nothing and return (probably into
                // the timer interrupt handler)
                return;
            }

            let mut next_task = maybe_next_task.unwrap();
            next_task.ticks += 1; // increment the tick count by 1
            next_task.status = ProcessStatus::Running;

            let new_stack_pointer = next_task.last_stack_pointer;
            let mut old_task = self.exchange_current_task(next_task);

            let task_ref = match old_task.status {
                ProcessStatus::Running => {
                    old_task.status = ProcessStatus::Ready;
                    self.ready_queue.push_back(old_task);
                    self.ready_queue.back_mut().unwrap()
                }
                ProcessStatus::Finished => {
                    old_task.status = ProcessStatus::Invalid;
                    self.finished_tasks.push_back(old_task);
                    self.finished_tasks.back_mut().unwrap()
                }
                ProcessStatus::Sleeping => {
                    self.sleeping_tasks.insert(old_task.sleep_ticks, old_task)
                }
                _ => panic!("unexpected process status: {:?}", old_task.status),
            };

            switch_args = Some((
                &mut task_ref.last_stack_pointer as *mut usize,
                new_stack_pointer as *const usize,
            ));
        });

        if let Some((old_stack, new_stack)) = switch_args {
            unsafe {
                switch(old_stack, new_stack);
            }
        }
    }

    /// Replaces the current task with the given new task and returns the old one.
    fn exchange_current_task(&mut self, new_task: Task) -> Task {
        let mut tmp = new_task;
        swap(&mut tmp, &mut self.current_task);
        tmp
    }
}
