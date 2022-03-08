use crate::io::fs::path::{Path, SEPARATOR};
use alloc::vec::Vec;

pub struct Components<'a> {
    fragments: Vec<&'a str>,
    seen_root: bool,
    absolute: bool,
    index: usize,
}

impl<'a> Components<'a> {
    pub fn new(path: &'a Path) -> Self {
        let absolute = path.starts_with(SEPARATOR);
        Components {
            fragments: path.split(SEPARATOR).collect(),
            seen_root: false,
            absolute,
            index: 0,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Component<'a> {
    RootDir,
    CurrentDir,
    ParentDir,
    Normal(&'a str),
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.absolute && !self.seen_root {
            self.seen_root = true;
            return Some(Component::RootDir);
        }

        let mut item = "";
        while item.is_empty() {
            if self.index >= self.fragments.len() {
                return None;
            }

            // hello///world should be equivalent to hello/world, so we skip empty fragments
            item = self.fragments[self.index];
            self.index += 1;
        }

        match item {
            "." => Some(Component::CurrentDir),
            ".." => Some(Component::ParentDir),
            _ => Some(Component::Normal(item)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_components() {
        let p = Path::new("hello/world");
        let mut c = p.components();
        assert_eq!(Some(Component::Normal("hello")), c.next());
        assert_eq!(Some(Component::Normal("world")), c.next());
        assert_eq!(None, c.next());
    }

    #[test_case]
    fn test_components_absolute() {
        let p = Path::new("/hello/world");
        let mut c = p.components();
        assert_eq!(Some(Component::RootDir), c.next());
        assert_eq!(Some(Component::Normal("hello")), c.next());
        assert_eq!(Some(Component::Normal("world")), c.next());
        assert_eq!(None, c.next());
    }

    #[test_case]
    fn test_empty_fragments() {
        let p = Path::new("hello///world");
        let mut c = p.components();
        assert_eq!(Some(Component::Normal("hello")), c.next());
        assert_eq!(Some(Component::Normal("world")), c.next());
        assert_eq!(None, c.next());
    }
}
