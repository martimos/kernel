use alloc::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};
use core::time::Duration;
use core::{
    cell::RefCell,
    sync::atomic::{AtomicU32, Ordering},
};

use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

use crate::scheduler::reschedule;
use crate::scheduler::switch::switch;
use crate::{
    hlt_loop, info,
    scheduler::{
        task::{ProcessStatus, Task},
        tid::Tid,
    },
    Result,
};

type TaskHandle = Rc<RefCell<Task>>;

pub struct Scheduler {
    current_task: TaskHandle,
    idle_task: Option<TaskHandle>,
    /// Tasks that are ready to be scheduled.
    ready_queue: Mutex<VecDeque<TaskHandle>>,
    /// Finished tasks waiting for cleanup.
    finished_tasks: Mutex<VecDeque<Tid>>,
    _sleeping_tasks: Mutex<VecDeque<TaskHandle>>,
    /// Tasks by their Pid.
    tasks: Mutex<BTreeMap<Tid, TaskHandle>>,
    /// The amount of running or ready tasks.
    /// Finished tasks are not included.
    task_count: AtomicU32,
    ticks: u64,
}

impl !Default for Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        let current_tid = Tid::new();
        let current_task = Rc::new(RefCell::new(Task::new_for_current(current_tid)));

        let tid = Tid::new();
        let idle_task = Rc::new(RefCell::new(Task::new_idle(tid)));

        let tasks = Mutex::new(BTreeMap::new());
        tasks.lock().insert(tid, idle_task.clone());
        tasks.lock().insert(current_tid, current_task.clone());

        let ready_queue = Mutex::new(VecDeque::new());

        Self {
            current_task,
            idle_task: Some(idle_task),
            ready_queue,
            finished_tasks: Mutex::new(VecDeque::new()),
            _sleeping_tasks: Mutex::new(VecDeque::new()),
            tasks,
            task_count: AtomicU32::new(0),
            ticks: 0,
        }
    }

    #[cfg(debug_assertions)]
    pub fn disable_idle_task(&mut self) {
        self.idle_task = None;
    }

    pub fn spawn(&mut self, func: extern "C" fn()) -> Result<Tid> {
        without_interrupts(|| {
            // Create the new task.
            let tid = Tid::new();
            let task = Rc::new(RefCell::new(Task::new(tid, ProcessStatus::Ready)));

            task.borrow_mut().allocate_stack(func);

            // Add it to the task lists.
            self.ready_queue.lock().push_back(task.clone());
            self.tasks.lock().insert(tid, task);
            self.task_count.fetch_add(1, Ordering::SeqCst);

            Ok(tid)
        })
    }

    pub fn cpu_time(&mut self) -> Duration {
        // TODO: currently, it feels like the interrupts occur in 100ms intervals, so use that, but it's probably inaccurate
        Duration::from_millis(self.current_task.borrow().ticks * 100)
    }

    pub fn join(&mut self, tid: Tid) {
        without_interrupts(|| {
            if tid == self.get_current_tid() {
                // don't deadlock ourselves
                return;
            }

            while self.tasks.lock().contains_key(&tid) {
                reschedule();
            }
        })
    }

    /// Terminates the currently running task and reschedules,
    /// so that the next available task will be run.
    pub fn exit(&mut self) -> ! {
        without_interrupts(|| {
            // serial_println!(
            //     "marking task {} to be finished",
            //     self.current_task.borrow().tid
            // );
            self.current_task.borrow_mut().status = ProcessStatus::Finished;
            self.task_count.fetch_sub(1, Ordering::SeqCst);
        });

        self.reschedule();
        hlt_loop() // just hlt until this is finally collected
    }

    /// Returns the task id (tid) of the currently running task.
    pub fn get_current_tid(&self) -> Tid {
        without_interrupts(|| self.current_task.borrow().tid)
    }

    pub fn total_ticks(&self) -> u64 {
        self.ticks
    }

    pub fn timer_tick(&mut self) {
        self.ticks += 1
    }

    pub fn reschedule(&mut self) {
        // If there are finished tasks waiting for deallocation, we do that first.
        // One task cleanup per schedule should on average be enough (hopefully)
        // to not accumulate a whole pile of finished, not cleaned up, tasks.

        let mut switch_args: Option<(*mut usize, usize)> = None;

        without_interrupts(|| {
            while let Some(id) = self.finished_tasks.lock().pop_front() {
                self.tasks
                    .lock()
                    .remove(&id)
                    .expect("finished task must be part of the task list");
            }

            let current_tid: Tid;
            let current_stack_pointer: *mut usize;
            let current_status: ProcessStatus;
            {
                let mut borrowed = self.current_task.borrow_mut();
                current_tid = borrowed.tid;
                current_stack_pointer = &mut borrowed.last_stack_pointer as *mut usize;
                current_status = borrowed.status;
            }

            // TODO: create tests for this
            let mut next_task = match current_status {
                ProcessStatus::Running => self.ready_queue.lock().pop_front(),
                _ => self.ready_queue.lock().pop_front(),
            };

            if next_task.is_none() {
                if self.idle_task.is_none() {
                    info!("no more tasks to run (not even an idle task)");
                    return;
                }
                next_task = Some(self.idle_task.as_ref().unwrap().clone());
            }

            if let Some(task) = next_task {
                task.borrow_mut().ticks += 1; // increment the tick count by 1

                // extract the Tid and the stack pointer from the next task
                let (new_id, new_stack_pointer) = {
                    let mut borrowed = task.borrow_mut();
                    borrowed.status = ProcessStatus::Running;
                    (borrowed.tid, borrowed.last_stack_pointer)
                };

                // if the next task is the idle task, do nothing
                if let Some(tid) = self.idle_task.as_ref().map(|t| t.borrow().tid) {
                    if tid == new_id {
                        /*
                        We don't need to hlt() here.
                        If this was called from an interrupt handler, it will return into that
                        handler, and that's it. If this was called from somewhere else, the caller
                        immediately continues to execute code.
                         */
                        return;
                    }
                }

                if current_status == ProcessStatus::Running {
                    // info!("task {} is ready", current_tid);
                    self.current_task.borrow_mut().status = ProcessStatus::Ready;
                    self.ready_queue.lock().push_back(self.current_task.clone());
                } else if current_status == ProcessStatus::Finished {
                    // info!("task {} is finished", current_tid);
                    self.current_task.borrow_mut().status = ProcessStatus::Invalid;
                    // release the task later, because the stack is required
                    // to call the function "switch"
                    self.finished_tasks.lock().push_back(current_tid);
                }

                // info!(
                //     "switch from tid:{} to tid:{} (*stack: {:#X}, {:#X})",
                //     current_tid,
                //     new_id,
                //     unsafe { *current_stack_pointer },
                //     new_stack_pointer,
                // );

                self.current_task = task;

                switch_args = Some((current_stack_pointer, new_stack_pointer));
            }
        });

        if let Some(args) = switch_args {
            unsafe {
                switch(args.0, args.1);
            }
        }
    }
}
