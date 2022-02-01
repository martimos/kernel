use crate::scheduler;
use crate::scheduler::pid::Pid;
use crate::syscall::error::Result;

pub fn getpid() -> Result<Pid> {
    Ok(scheduler::get_current_pid())
}
