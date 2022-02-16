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
        const O_RDONLY = 1 << 0;
        const O_WRONLY = 1 << 1;
        const O_RDWR = 1 << 2;
        const O_APPEND = 1 << 3;
        const O_CREAT = 1 << 4;
        const O_DSYNC = 1 << 5;
        const O_EXCL = 1 << 6;
        const O_NOCTTY = 1 << 7;
        const O_NONBLOCK = 1 << 8;
        const O_RSYNC = 1 << 9;
        const O_SYNC = 1 << 10;
        const O_TRUNC = 1 << 11;
    }
}
