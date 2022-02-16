use crate::filesystem::path::components::{Component, Components};
use crate::filesystem::path::{Path, SEPARATOR};
use alloc::string::String;
use core::borrow::Borrow;
use core::fmt::{Display, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub struct OwnedPath {
    inner: String,
}

impl<P: AsRef<Path>> From<P> for OwnedPath {
    fn from(v: P) -> Self {
        let mut s = Self::new();
        s.push(v);
        s
    }
}

impl Borrow<Path> for OwnedPath {
    fn borrow(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl OwnedPath {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    pub fn push<P: AsRef<Path>>(&mut self, segment: P) {
        let path = segment.as_ref();

        if self.len() == 0 || self.inner.chars().last().unwrap() != SEPARATOR {
            // we need to push a separator if the path is empty or
            // if the rightmost char is not a separator
            self.inner.push(SEPARATOR)
        }

        path.components().for_each(|c| {
            match c {
                Component::CurrentDir => { /* do nothing here */ }
                Component::ParentDir => todo!("remove last"),
                Component::Normal(s) => self.inner.push_str(s),
                Component::RootDir => { /* do nothing here */ }
            }
        });
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn components(&self) -> Components<'_> {
        Path::new(&self.inner).components()
    }
}

impl Display for OwnedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test_case]
    fn test_push_trivial() {
        let mut p = OwnedPath::new();
        p.push("hello");
        p.push("world");
        assert_eq!("/hello/world", p.to_string());
    }

    #[test_case]
    fn test_push_separators() {
        let mut p = OwnedPath::new();
        p.push("hello");
        p.push("world/");
        assert_eq!("/hello/world", p.to_string());
    }
}
