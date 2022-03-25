use bitflags::bitflags;

bitflags! {
    pub struct Permission: u16 {
        const STICKY = 1 << 9;

        const USER_READ = 1 << 8;
        const USER_WRITE = 1 << 7;
        const USER_EXECUTE = 1 << 6;

        const GROUP_READ = 1 << 5;
        const GROUP_WRITE = 1 << 4;
        const GROUP_EXECUTE = 1 << 3;

        const OTHER_READ = 1 << 2;
        const OTHER_WRITE = 1 << 1;
        const OTHER_EXECUTE = 1 << 0;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Triad {
    User,
    Group,
    Other,
}

impl Triad {
    fn get_read_perm(&self) -> Permission {
        match self {
            Triad::User => Permission::USER_READ,
            Triad::Group => Permission::GROUP_READ,
            Triad::Other => Permission::OTHER_READ,
        }
    }

    fn get_write_perm(&self) -> Permission {
        match self {
            Triad::User => Permission::USER_WRITE,
            Triad::Group => Permission::GROUP_WRITE,
            Triad::Other => Permission::OTHER_WRITE,
        }
    }

    fn get_execute_perm(&self) -> Permission {
        match self {
            Triad::User => Permission::USER_EXECUTE,
            Triad::Group => Permission::GROUP_EXECUTE,
            Triad::Other => Permission::OTHER_EXECUTE,
        }
    }
}

impl Default for Permission {
    fn default() -> Self {
        Self::empty()
    }
}

impl Permission {
    pub fn can_read(&self, triad: Triad) -> bool {
        self.contains(triad.get_read_perm())
    }

    pub fn can_write(&self, triad: Triad) -> bool {
        self.contains(triad.get_write_perm())
    }

    pub fn can_execute(&self, triad: Triad) -> bool {
        self.contains(triad.get_execute_perm())
    }

    pub fn is_sticky(&self) -> bool {
        self.contains(Permission::STICKY)
    }

    pub fn set_readable(&mut self, triad: Triad, readable: bool) {
        self.set(triad.get_read_perm(), readable)
    }

    pub fn set_writable(&mut self, triad: Triad, writable: bool) {
        self.set(triad.get_write_perm(), writable)
    }

    pub fn set_executable(&mut self, triad: Triad, executable: bool) {
        self.set(triad.get_execute_perm(), executable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_permission_default_empty() {
        let perm = Permission::default();
        assert_eq!(Permission::empty(), perm);
    }

    #[test_case]
    fn test_empty_permissions() {
        let perm = Permission::empty();

        assert!(!perm.can_read(Triad::User));
        assert!(!perm.can_read(Triad::Group));
        assert!(!perm.can_read(Triad::Other));

        assert!(!perm.can_write(Triad::User));
        assert!(!perm.can_write(Triad::Group));
        assert!(!perm.can_write(Triad::Other));

        assert!(!perm.can_execute(Triad::User));
        assert!(!perm.can_execute(Triad::Group));
        assert!(!perm.can_execute(Triad::Other));
    }

    #[test_case]
    fn test_triad_get_permission() {
        assert_eq!(Permission::USER_READ, Triad::User.get_read_perm());
        assert_eq!(Permission::USER_WRITE, Triad::User.get_write_perm());
        assert_eq!(Permission::USER_EXECUTE, Triad::User.get_execute_perm());

        assert_eq!(Permission::GROUP_READ, Triad::Group.get_read_perm());
        assert_eq!(Permission::GROUP_WRITE, Triad::Group.get_write_perm());
        assert_eq!(Permission::GROUP_EXECUTE, Triad::Group.get_execute_perm());

        assert_eq!(Permission::OTHER_READ, Triad::Other.get_read_perm());
        assert_eq!(Permission::OTHER_WRITE, Triad::Other.get_write_perm());
        assert_eq!(Permission::OTHER_EXECUTE, Triad::Other.get_execute_perm());
    }

    #[test_case]
    fn test_set_readable() {
        let mut perm = Permission::empty();

        for triad in [Triad::User, Triad::Group, Triad::Other] {
            perm.set_readable(triad, true);
            assert_eq!(triad.get_read_perm(), perm);

            perm.set_readable(triad, false);
            assert_eq!(Permission::empty(), perm);
        }
    }

    #[test_case]
    fn test_set_writable() {
        let mut perm = Permission::empty();

        for triad in [Triad::User, Triad::Group, Triad::Other] {
            perm.set_writable(triad, true);
            assert_eq!(triad.get_write_perm(), perm);

            perm.set_writable(triad, false);
            assert_eq!(Permission::empty(), perm);
        }
    }

    #[test_case]
    fn test_set_executable() {
        let mut perm = Permission::empty();

        for triad in [Triad::User, Triad::Group, Triad::Other] {
            perm.set_executable(triad, true);
            assert_eq!(triad.get_execute_perm(), perm);

            perm.set_executable(triad, false);
            assert_eq!(Permission::empty(), perm);
        }
    }

    #[test_case]
    fn test_from_u16() {
        let perm = Permission::from_bits_truncate(0o755);
        assert!(perm.can_read(Triad::User));
        assert!(perm.can_write(Triad::User));
        assert!(perm.can_execute(Triad::User));

        assert!(perm.can_read(Triad::Group));
        assert!(!perm.can_write(Triad::Group));
        assert!(perm.can_execute(Triad::Group));

        assert!(perm.can_read(Triad::Other));
        assert!(!perm.can_write(Triad::Other));
        assert!(perm.can_execute(Triad::Other));
    }
}
