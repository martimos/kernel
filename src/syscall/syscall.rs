use crate::{scheduler, scheduler::pid::Pid, syscall::error::Result};

pub fn getpid() -> Result<Pid> {
    Ok(scheduler::get_current_pid())
}
