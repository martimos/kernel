use bitflags::bitflags;

bitflags! {
    pub struct MountFlags: u32 {
        const NONE = 1 << 0;
        const NOEXEC = 1 << 1;
        const READONLY = 1 << 2;
        const SYNCHRONOUS = 1 << 3;
    }
}

bitflags! {
    pub struct OpenFlags: u32 { // probably not the correct numeric values
        /// Open read-only.
        const O_RDONLY = 1 << 0;
        /// Open write-only.
        const O_WRONLY = 1 << 1;
        /// Open read-write.
        const O_RDWR = 1 << 2;
        /// Writes append to the file.
        const O_APPEND = 1 << 3;
        /// Create if it doesn't exist.
        const O_CREAT = 1 << 4;
        /// Synchronize data.
        const O_DSYNC = 1 << 5;
        /// Fail if the file exists.
        const O_EXCL = 1 << 6;
        /// Don't assign a controlling terminal.
        const O_NOCTTY = 1 << 7;
        /// Non-blocking I/O.
        const O_NONBLOCK = 1 << 8;
        /// Synchronize read operations.
        const O_RSYNC = 1 << 9;
        /// Synchronous writes.
        const O_SYNC = 1 << 10;
        /// Truncate the file to length 0.
        const O_TRUNC = 1 << 11;
    }
}
