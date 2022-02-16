use crate::filesystem::path::components::Components;
use crate::filesystem::path::owned::OwnedPath;
use alloc::borrow::ToOwned;
use core::fmt::{Display, Formatter};
use core::ops::Deref;

pub mod components;
pub mod owned;

pub const SEPARATOR: char = '/';

pub fn is_separator_char(c: char) -> bool {
    c == SEPARATOR
}

pub struct Path {
    inner: str,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Path {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    pub fn components(&self) -> Components<'_> {
        Components::new(self)
    }
}

impl ToOwned for Path {
    type Owned = OwnedPath;

    fn to_owned(&self) -> Self::Owned {
        self.into()
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test_case]
    fn test_foo() {
        let p = Path::new("/hello");
        assert_eq!("/hello", p.to_string());
    }
}
