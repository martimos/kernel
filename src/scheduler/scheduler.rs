use crate::scheduler::pid::Pid;
use crate::scheduler::priority::Priority;
use crate::scheduler::queue::PriorityTaskQueue;
use crate::scheduler::switch::switch;
use crate::scheduler::task::{ProcessStatus, Task};
use crate::scheduler::NO_PRIORITIES;
use crate::serial_println;
use crate::syscall::error::Errno;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

type TaskHandle = Rc<RefCell<Task>>;

pub struct Scheduler {
    current_task: TaskHandle,
    idle_task: TaskHandle,
    /// Tasks that are ready to be scheduled.
    ready_queue: Mutex<PriorityTaskQueue>,
    /// Finished tasks waiting for cleanup.
    finished_tasks: Mutex<VecDeque<Pid>>,
    /// Tasks by their Pid.
    tasks: Mutex<BTreeMap<Pid, TaskHandle>>,
    /// The amount of running or ready tasks.
    /// Finished tasks are not included.
    task_count: AtomicU32,
}

impl Scheduler {
    pub fn new() -> Self {
        let pid = Pid::new();
        let idle_task = Rc::new(RefCell::new(Task::new_idle(pid)));
        let tasks = Mutex::new(BTreeMap::new());

        tasks.lock().insert(pid, idle_task.clone());

        Self {
            current_task: idle_task.clone(),
            idle_task: idle_task.clone(),
            ready_queue: Mutex::new(PriorityTaskQueue::new()),
            finished_tasks: Mutex::new(VecDeque::<Pid>::new()),
            tasks,
            task_count: AtomicU32::new(0),
        }
    }

    pub fn spawn(&mut self, func: extern "C" fn(), prio: Priority) -> Result<Pid, Errno> {
        without_interrupts(|| {
            let prio_number = prio.as_u8() as usize;
            if prio_number >= NO_PRIORITIES {
                return Err(Errno::EINVAL);
            }

            // Create the new task.
            let tid = Pid::new();
            let task = Rc::new(RefCell::new(Task::new(tid, ProcessStatus::Ready, prio)));

            task.borrow_mut().allocate_stack(func);

            // Add it to the task lists.
            self.ready_queue.lock().push(task.clone());
            self.tasks.lock().insert(tid, task);
            self.task_count.fetch_add(1, Ordering::SeqCst);

            Ok(tid)
        })
    }

    pub fn exit(&mut self) -> ! {
        without_interrupts(|| {
            if self.current_task.borrow().status != ProcessStatus::Idle {
                serial_println!(
                    "marking task {} to be finished",
                    self.current_task.borrow().pid
                );
                self.current_task.borrow_mut().status = ProcessStatus::Finished;
                self.task_count.fetch_sub(1, Ordering::SeqCst);
            } else {
                panic!("unable to terminate idle task");
            }
        });

        self.reschedule();

        panic!("exit returned")
    }

    pub fn abort(&mut self) -> ! {
        without_interrupts(|| {
            if self.current_task.borrow().status != ProcessStatus::Idle {
                serial_println!("abort task with id {}", self.current_task.borrow().pid);
                self.current_task.borrow_mut().status = ProcessStatus::Finished;
                self.task_count.fetch_sub(1, Ordering::SeqCst);
            } else {
                panic!("unable to terminate idle task");
            }
        });

        self.reschedule();

        panic!("abort returned")
    }

    pub fn block_current_task(&mut self) -> Rc<RefCell<Task>> {
        without_interrupts(|| {
            if self.current_task.borrow().status == ProcessStatus::Running {
                serial_println!("block task {}", self.current_task.borrow().pid);

                self.current_task.borrow_mut().status = ProcessStatus::Blocked;
                self.current_task.clone()
            } else {
                panic!("unable to block task {}", self.current_task.borrow().pid);
            }
        })
    }

    pub fn wakeup_task(&mut self, task: TaskHandle) {
        without_interrupts(|| {
            if task.borrow().status == ProcessStatus::Blocked {
                serial_println!("wake up task {}", task.borrow().pid);

                task.borrow_mut().status = ProcessStatus::Ready;
                self.ready_queue.lock().push(task.clone());
            }
        });
    }

    pub fn get_current_pid(&self) -> Pid {
        without_interrupts(|| self.current_task.borrow().pid)
    }

    pub fn get_current_stack_bottom(&self) -> usize {
        without_interrupts(|| unsafe { (*self.current_task.borrow().stack).bottom() })
    }

    pub fn schedule(&mut self) {
        // If there are finished tasks waiting for deallocation, we do that first.
        // One task cleanup per schedule should on average be enough (hopefully)
        // to not accumulate a whole pile of finished, not cleaned up, tasks.

        if let Some(id) = self.finished_tasks.lock().pop_front() {
            self.tasks
                .lock()
                .remove(&id)
                .expect("finished task must be part of the task list");
        }

        let current_pid: Pid;
        let current_stack_pointer: *mut usize;
        let current_prio: Priority;
        let current_status: ProcessStatus;
        {
            let mut borrowed = self.current_task.borrow_mut();
            current_pid = borrowed.pid;
            current_stack_pointer = &mut borrowed.last_stack_pointer as *mut usize;
            current_prio = borrowed.prio;
            current_status = borrowed.status;
        }

        let mut next_task = match current_status {
            ProcessStatus::Running => self.ready_queue.lock().pop_with_prio(current_prio),
            _ => self.ready_queue.lock().pop(),
        };

        if next_task.is_none() {
            if current_status != ProcessStatus::Running && current_status != ProcessStatus::Idle {
                serial_println!("next task is idle task");
                next_task = Some(self.idle_task.clone());
            }
        }

        if let Some(task) = next_task {
            let (new_id, new_stack_pointer) = {
                let mut borrowed = task.borrow_mut();
                borrowed.status = ProcessStatus::Running;
                (borrowed.pid, borrowed.last_stack_pointer)
            };

            if current_status == ProcessStatus::Running {
                serial_println!("task {} is ready", current_pid);
                self.current_task.borrow_mut().status = ProcessStatus::Ready;
                self.ready_queue.lock().push(self.current_task.clone());
            } else if current_status == ProcessStatus::Finished {
                serial_println!("task {} is finished", current_pid);
                self.current_task.borrow_mut().status = ProcessStatus::Invalid;
                // release the task later, because the stack is required
                // to call the function "switch"
                self.finished_tasks.lock().push_back(current_pid);
            }

            serial_println!(
                "switch from pid:{} to pid:{} (*stack: {:#X}, {:#X})",
                current_pid,
                new_id,
                unsafe { *current_stack_pointer },
                new_stack_pointer,
            );

            self.current_task = task;

            unsafe {
                switch(current_stack_pointer, new_stack_pointer);
            }
        }
    }

    pub fn reschedule(&mut self) {
        without_interrupts(|| self.schedule());
    }
}
