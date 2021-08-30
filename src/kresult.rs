use crate::syscall::error::Errno;

pub type KResult<T> = Result<T, Errno>;
