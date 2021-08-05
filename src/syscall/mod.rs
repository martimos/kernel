use core::convert::TryInto;

use error::Result;

use crate::syscall::error::Errno;
use crate::vga_println;

pub mod error;

#[allow(clippy::module_inception)]
mod syscall;

pub fn syscall(eax: usize, _ebx: usize, _edx: usize, _esi: usize, _edi: usize) -> usize {
    match eax.try_into() {
        Ok(Syscall::Getpid) => getpid(),
        Ok(_) => Errno::ENOSYS.into(), // syscall recognized but not implemented
        Err(_) => {
            vga_println!("unrecognized syscall {}", eax);
            Errno::ENOSYS.into()
        }
    }
}

pub fn getpid() -> usize {
    match syscall::getpid() {
        Ok(res) => res.into(),
        Err(errno) => errno as usize,
    }
}

/// Generates a TryFrom implementation for the enclosed enum.
///
///     generate_try_from! {
///         enum MyEnum: usize {
///             ...
///         }
///     }
///
/// will generate a TryFrom&lt;usize&gt; implementation for MyEnum.
macro_rules! generate_try_from {
    ($(#[$meta:meta])* $vis:vis enum $name:ident : $typ:ty {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl core::convert::TryFrom<$typ> for $name {
            type Error = ();

            fn try_from(v: $typ) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as $typ => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

generate_try_from! {
    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    #[repr(usize)]
    pub enum Syscall: usize {
        Read = 0,
        Write = 1,
        Open = 2,
        Close = 3,
        Stat = 4,
        Fstat = 5,
        Lstat = 6,
        Poll = 7,
        Lseek = 8,
        Mmap = 9,
        Mprotect = 10,
        Munmap = 11,
        Brk = 12,
        RtSigaction = 13,
        RtSigprocmask = 14,
        RtSigreturn = 15,
        Ioctl = 16,
        Pread64 = 17,
        Pwrite64 = 18,
        Readv = 19,
        Writev = 20,
        Access = 21,
        Pipe = 22,
        Select = 23,
        SchedYield = 24,
        Mremap = 25,
        Msync = 26,
        Mincore = 27,
        Madvise = 28,
        Shmget = 29,
        Shmat = 30,
        Shmctl = 31,
        Dup = 32,
        Dup2 = 33,
        Pause = 34,
        Nanosleep = 35,
        Getitimer = 36,
        Alarm = 37,
        Setitimer = 38,
        Getpid = 39,
        Sendfile = 40,
        Socket = 41,
        Connect = 42,
        Accept = 43,
        Sendto = 44,
        Recvfrom = 45,
        Sendmsg = 46,
        Recvmsg = 47,
        Shutdown = 48,
        Bind = 49,
        Listen = 50,
        Getsockname = 51,
        Getpeername = 52,
        Socketpair = 53,
        Setsockopt = 54,
        Getsockopt = 55,
        Clone = 56,
        Fork = 57,
        Vfork = 58,
        Execve = 59,
        Exit = 60,
        Wait4 = 61,
        Kill = 62,
        Uname = 63,
        Semget = 64,
        Semop = 65,
        Semctl = 66,
        Shmdt = 67,
        Msgget = 68,
        Msgsnd = 69,
        Msgrcv = 70,
        Msgctl = 71,
        Fcntl = 72,
        Flock = 73,
        Fsync = 74,
        Fdatasync = 75,
        Truncate = 76,
        Ftruncate = 77,
        Getdents = 78,
        Getcwd = 79,
        Chdir = 80,
        Fchdir = 81,
        Rename = 82,
        Mkdir = 83,
        Rmdir = 84,
        Creat = 85,
        Link = 86,
        Unlink = 87,
        Symlink = 88,
        Readlink = 89,
        Chmod = 90,
        Fchmod = 91,
        Chown = 92,
        Fchown = 93,
        Lchown = 94,
        Umask = 95,
        Gettimeofday = 96,
        Getrlimit = 97,
        Getrusage = 98,
        Sysinfo = 99,
        Times = 100,
        Ptrace = 101,
        Getuid = 102,
        Syslog = 103,
        Getgid = 104,
        Setuid = 105,
        Setgid = 106,
        Geteuid = 107,
        Getegid = 108,
        Setpgid = 109,
        Getppid = 110,
        Getpgrp = 111,
        Setsid = 112,
        Setreuid = 113,
        Setregid = 114,
        Getgroups = 115,
        Setgroups = 116,
        Setresuid = 117,
        Getresuid = 118,
        Setresgid = 119,
        Getresgid = 120,
        Getpgid = 121,
        Setfsuid = 122,
        Setfsgid = 123,
        Getsid = 124,
        Capget = 125,
        Capset = 126,
        RtSigpending = 127,
        RtSigtimedwait = 128,
        RtSigqueueinfo = 129,
        RtSigsuspend = 130,
        Sigaltstack = 131,
        Utime = 132,
        Mknod = 133,
        Uselib = 134,
        Personality = 135,
        Ustat = 136,
        Statfs = 137,
        Fstatfs = 138,
        Sysfs = 139,
        Getpriority = 140,
        Setpriority = 141,
        SchedSetparam = 142,
        SchedGetparam = 143,
        SchedSetscheduler = 144,
        SchedGetscheduler = 145,
        SchedGetPriorityMax = 146,
        SchedGetPriorityMin = 147,
        SchedRrGetInterval = 148,
        Mlock = 149,
        Munlock = 150,
        Mlockall = 151,
        Munlockall = 152,
        Vhangup = 153,
        ModifyLdt = 154,
        PivotRoot = 155,
        Sysctl = 156,
        Prctl = 157,
        ArchPrctl = 158,
        Adjtimex = 159,
        Setrlimit = 160,
        Chroot = 161,
        Sync = 162,
        Acct = 163,
        Settimeofday = 164,
        Mount = 165,
        Umount2 = 166,
        Swapon = 167,
        Swapoff = 168,
        Reboot = 169,
        Sethostname = 170,
        Setdomainname = 171,
        Iopl = 172,
        Ioperm = 173,
        CreateModule = 174,
        InitModule = 175,
        DeleteModule = 176,
        GetKernelSyms = 177,
        QueryModule = 178,
        Quotactl = 179,
        Nfsservctl = 180,
        Getpmsg = 181,
        Putpmsg = 182,
        AfsSyscall = 183,
        Tuxcall = 184,
        Security = 185,
        Gettid = 186,
        Readahead = 187,
        Setxattr = 188,
        Lsetxattr = 189,
        Fsetxattr = 190,
        Getxattr = 191,
        Lgetxattr = 192,
        Fgetxattr = 193,
        Listxattr = 194,
        Llistxattr = 195,
        Flistxattr = 196,
        Removexattr = 197,
        Lremovexattr = 198,
        Fremovexattr = 199,
        Tkill = 200,
        Time = 201,
        Futex = 202,
        SchedSetaffinity = 203,
        SchedGetaffinity = 204,
        SetThreadArea = 205,
        IoSetup = 206,
        IoDestroy = 207,
        IoGetevents = 208,
        IoSubmit = 209,
        IoCancel = 210,
        GetThreadArea = 211,
        LookupDcookie = 212,
        EpollCreate = 213,
        EpollCtlOld = 214,
        EpollWaitOld = 215,
        RemapFilePages = 216,
        Getdents64 = 217,
        SetTidAddress = 218,
        RestartSyscall = 219,
        Semtimedop = 220,
        Fadvise64 = 221,
        TimerCreate = 222,
        TimerSettime = 223,
        TimerGettime = 224,
        TimerGetoverrun = 225,
        TimerDelete = 226,
        ClockSettime = 227,
        ClockGettime = 228,
        ClockGetres = 229,
        ClockNanosleep = 230,
        ExitGroup = 231,
        EpollWait = 232,
        EpollCtl = 233,
        Tgkill = 234,
        Utimes = 235,
        Vserver = 236,
        Mbind = 237,
        SetMempolicy = 238,
        GetMempolicy = 239,
        MqOpen = 240,
        MqUnlink = 241,
        MqTimedsend = 242,
        MqTimedreceive = 243,
        MqNotify = 244,
        MqGetsetattr = 245,
        KexecLoad = 246,
        Waitid = 247,
        AddKey = 248,
        RequestKey = 249,
        Keyctl = 250,
        IoprioSet = 251,
        IoprioGet = 252,
        InotifyInit = 253,
        InotifyAddWatch = 254,
        InotifyRmWatch = 255,
        MigratePages = 256,
        Openat = 257,
        Mkdirat = 258,
        Mknodat = 259,
        Fchownat = 260,
        Futimesat = 261,
        Newfstatat = 262,
        Unlinkat = 263,
        Renameat = 264,
        Linkat = 265,
        Symlinkat = 266,
        Readlinkat = 267,
        Fchmodat = 268,
        Faccessat = 269,
        Pselect6 = 270,
        Ppoll = 271,
        Unshare = 272,
        SetRobustList = 273,
        GetRobustList = 274,
        Splice = 275,
        Tee = 276,
        SyncFileRange = 277,
        Vmsplice = 278,
        MovePages = 279,
        Utimensat = 280,
        EpollPwait = 281,
        Signalfd = 282,
        TimerfdCreate = 283,
        Eventfd = 284,
        Fallocate = 285,
        TimerfdSettime = 286,
        TimerfdGettime = 287,
        Accept4 = 288,
        Signalfd4 = 289,
        Eventfd2 = 290,
        EpollCreate1 = 291,
        Dup3 = 292,
        Pipe2 = 293,
        InotifyInit1 = 294,
        Preadv = 295,
        Pwritev = 296,
        RtTgsigqueueinfo = 297,
        PerfEventOpen = 298,
        Recvmmsg = 299,
        FanotifyInit = 300,
        FanotifyMark = 301,
        Prlimit64 = 302,
        NameToHandleAt = 303,
        OpenByHandleAt = 304,
        ClockAdjtime = 305,
        Syncfs = 306,
        Sendmmsg = 307,
        Setns = 308,
        Getcpu = 309,
        ProcessVmReadv = 310,
        ProcessVmWritev = 311,
        Kcmp = 312,
        FinitModule = 313,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_try_from_syscall() {
        assert_eq!(Syscall::Read, 0_usize.try_into().unwrap());
        assert_eq!(Syscall::Write, 1_usize.try_into().unwrap());
        assert_eq!(Syscall::Getpid, 39_usize.try_into().unwrap());
        assert_eq!(Syscall::GetThreadArea, 211_usize.try_into().unwrap());
        assert_eq!(Syscall::FinitModule, 313_usize.try_into().unwrap());

        assert!(TryInto::<Syscall>::try_into(314_usize).is_err());
        assert!(TryInto::<Syscall>::try_into(999_usize).is_err());
    }
}
