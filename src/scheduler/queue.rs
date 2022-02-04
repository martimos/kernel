use crate::scheduler::{priority::Priority, task::Task, NUM_PRIORITIES};
use alloc::rc::Rc;
use core::{arch::asm, cell::RefCell};

struct QueueHead {
    head: Option<Rc<RefCell<Task>>>,
    tail: Option<Rc<RefCell<Task>>>,
}

impl QueueHead {
    pub const fn new() -> Self {
        QueueHead {
            head: None,
            tail: None,
        }
    }
}

impl Default for QueueHead {
    fn default() -> Self {
        Self::new()
    }
}

/// Realize a priority queue for tasks
pub struct PriorityTaskQueue {
    queues: [QueueHead; NUM_PRIORITIES],
    prio_bitmap: u64,
}

impl PriorityTaskQueue {
    /// Creates an empty priority queue for tasks
    pub fn new() -> PriorityTaskQueue {
        PriorityTaskQueue {
            queues: Default::default(),
            prio_bitmap: 0,
        }
    }

    /// Add a task by its priority to the queue
    pub fn push(&mut self, task: Rc<RefCell<Task>>) {
        let i: usize = task.borrow().prio.as_u8() as usize;

        self.prio_bitmap |= 1 << i;
        match self.queues[i].tail {
            Some(ref mut tail) => {
                // add task at the end of the node
                tail.borrow_mut().next = Some(task.clone());

                let mut borrow = task.borrow_mut();
                borrow.next = None;
                borrow.prev = Some(tail.clone());
            }
            None => {
                // first element in the queue
                self.queues[i].head = Some(task.clone());

                let mut borrow = task.borrow_mut();
                borrow.next = None;
                borrow.prev = None;
            }
        }

        self.queues[i].tail = Some(task.clone());
    }

    fn pop_from_queue(&mut self, queue_index: usize) -> Option<Rc<RefCell<Task>>> {
        let new_head;
        let task;

        match self.queues[queue_index].head {
            None => {
                return None;
            }
            Some(ref mut head) => {
                let mut borrow = head.borrow_mut();

                match borrow.next {
                    Some(ref mut nhead) => {
                        nhead.borrow_mut().prev = None;
                    }
                    None => {}
                }

                new_head = borrow.next.clone();
                borrow.next = None;
                borrow.prev = None;

                task = head.clone();
            }
        }

        self.queues[queue_index].head = new_head;
        if self.queues[queue_index].head.is_none() {
            self.queues[queue_index].tail = None;
            self.prio_bitmap &= !(1 << queue_index as u64);
        }

        Some(task)
    }

    /// Pop the task with the highest priority from the queue
    pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
        if let Some(i) = msb(self.prio_bitmap) {
            return self.pop_from_queue(i as usize);
        }

        None
    }

    /// Pop the next task, which has a higher or the same priority as `prio`
    pub fn pop_with_prio(&mut self, prio: Priority) -> Option<Rc<RefCell<Task>>> {
        if let Some(i) = msb(self.prio_bitmap) {
            if i >= prio.as_u8() as u64 {
                return self.pop_from_queue(i as usize);
            }
        }

        None
    }

    /// Remove a specific task from the priority queue.
    pub fn remove(&mut self, task: Rc<RefCell<Task>>) {
        let i = task.borrow().prio.as_u8() as usize;
        //assert!(i < NO_PRIORITIES, "Priority {} is too high", i);

        let mut curr = self.queues[i].head.clone();
        let mut next_curr;

        loop {
            match curr {
                Some(ref curr_task) => {
                    if Rc::ptr_eq(&curr_task, &task) {
                        let (mut prev, mut next) = {
                            let borrowed = curr_task.borrow_mut();
                            (borrowed.prev.clone(), borrowed.next.clone())
                        };

                        match prev {
                            Some(ref mut t) => {
                                t.borrow_mut().next = next.clone();
                            }
                            None => {}
                        };

                        match next {
                            Some(ref mut t) => {
                                t.borrow_mut().prev = prev.clone();
                            }
                            None => {}
                        };

                        break;
                    }

                    next_curr = curr_task.borrow().next.clone();
                }
                None => {
                    break;
                }
            }

            curr = next_curr.clone();
        }

        if let Some(ref curr_task) = self.queues[i].head {
            if Rc::ptr_eq(&curr_task, &task) {
                self.queues[i].head = task.borrow().next.clone();

                if self.queues[i].head.is_none() {
                    self.prio_bitmap &= !(1 << i as u64);
                }
            }
        }
    }
}

#[inline(always)]
pub fn msb(value: u64) -> Option<u64> {
    if value > 0 {
        let ret: u64;
        unsafe {
            asm!("bsr {0}, {1}",
            out(reg) ret,
            in(reg) value,
            options(nomem, nostack)
            );
        }
        Some(ret)
    } else {
        None
    }
}
