use crate::{scheduler, scheduler::tid::Tid, syscall::error::Result};

pub fn getpid() -> Result<Tid> {
    Ok(scheduler::get_current_tid())
}
