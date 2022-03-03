use crate::filesystem::path::components::Component;
use crate::filesystem::path::Path;
use crate::filesystem::vfs::vnode::{Type, VNode};
use crate::syscall::error::Errno;
use crate::{info, Result};
use alloc::borrow::ToOwned;

pub mod vnode;

pub fn init() {}

pub struct Vfs {
    root: Option<VNode>,
}

impl Vfs {
    pub const fn new() -> Self {
        Self { root: None }
    }

    pub fn find_vnode(&self, p: &dyn AsRef<Path>) -> Result<&VNode> {
        let path = p.as_ref().to_owned(); // OwnedPaths are always canonical
        let mut components = path.components();

        // check that we got an absolute path
        let first = components.next();
        if first != Some(Component::RootDir) {
            info!("path must be absolute, but was '{}'", path);
            return Err(Errno::ENOENT);
        } else if first.is_none() {
            info!("path can't be empty");
            return Err(Errno::ENOENT);
        }

        // walk the tree and find the node
        let mut current = match self.root.as_ref() {
            None => return Err(Errno::ENOENT),
            Some(v) => v,
        };
        for component in components {
            match component {
                Component::RootDir => panic!("unexpected root dir in the middle of a path"),
                Component::CurrentDir | Component::ParentDir => panic!("path is not canonical"), // shouldn't happen with an OwnedPath
                Component::Normal(v) => {
                    match current.typ() {
                        Type::File { .. } => {
                            // we found a file but are not at the end of the path yet,
                            // so we require a directory
                            return Err(Errno::ENOENT);
                        }
                        Type::Directory { children } => {
                            match children.iter().find(|&c| c.name() == v) {
                                None => return Err(Errno::ENOENT),
                                Some(v) => current = v,
                            };
                        }
                    };
                }
            };
        }

        // we found the vnode
        Ok(current)
    }
}
