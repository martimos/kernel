use kernel_constants::syscall::error::Errno;

pub type Result<T, E = Errno> = core::result::Result<T, E>;
