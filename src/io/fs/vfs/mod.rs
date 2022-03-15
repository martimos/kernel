use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;

use spin::{Mutex, Once};

use crate::io::fs::path::components::Component;
use crate::io::fs::path::Path;
use crate::io::fs::perm::Permission;
use crate::io::fs::vfs::vnode::{Type, VNode};
use crate::syscall::error::Errno;
use crate::{info, Result};

pub mod vnode;

static mut VFS: Option<Mutex<Vfs>> = None;
static VFS_INIT: Once = Once::new();

pub fn init() {
    VFS_INIT.call_once(|| unsafe {
        VFS = Some(Mutex::new(Vfs::new()));
    });
}

fn get_vfs() -> &'static Mutex<Vfs> {
    unsafe { VFS.as_ref().expect("vfs is not initialized") }
}

#[must_use = "mounting a VNode may fail"]
pub fn mount(p: &dyn AsRef<Path>, node: VNode) -> Result<()> {
    get_vfs().lock().mount(p, Arc::new(Mutex::new(node)))
}

pub fn find_vnode(p: &dyn AsRef<Path>) -> Result<Arc<Mutex<VNode>>> {
    get_vfs().lock().find_vnode(p)
}

impl !Default for Vfs {}

pub struct Vfs {
    root: Arc<Mutex<VNode>>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            root: Arc::new(Mutex::new(VNode::new_directory(
                "/".into(),
                Permission::empty(),
                vec![],
            ))),
        }
    }

    fn mount(&mut self, p: &dyn AsRef<Path>, node: Arc<Mutex<VNode>>) -> Result<()> {
        let parent = self.find_vnode(p)?;
        let mut guard = parent.lock();
        let typ = guard.typ_mut();
        let children = match typ {
            Type::File { .. } => return Err(Errno::ENOTDIR),
            Type::Directory { children } => children,
        };
        let node_name = node.lock().name().to_string();
        if children.contains_key(&node_name) {
            return Err(Errno::EBUSY);
        }
        children.insert(node_name, node);

        Ok(())
    }

    fn find_vnode(&mut self, p: &dyn AsRef<Path>) -> Result<Arc<Mutex<VNode>>> {
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
        let mut current = self.root.clone();
        let mut seen_root = false;
        for component in components {
            match component {
                Component::RootDir => {
                    if seen_root {
                        panic!("unexpected root dir in the middle of a path");
                    }
                    seen_root = true;
                }
                Component::CurrentDir | Component::ParentDir => panic!("path is not canonical"), // shouldn't happen with an OwnedPath
                Component::Normal(v) => {
                    let guard = current.lock();
                    let new_current = match guard.typ() {
                        Type::File { .. } => {
                            // we found a file but are not at the end of the path yet,
                            // so we require a directory
                            return Err(Errno::ENOENT);
                        }
                        Type::Directory { children } => {
                            if !children.contains_key(v) {
                                return Err(Errno::ENOENT);
                            }
                            children.get(v).cloned().unwrap()
                        }
                    };
                    drop(guard);
                    current = new_current;
                }
            };
        }

        // we found the vnode
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use core::assert_eq;

    use crate::io::fs::devfs::DevFs;

    use super::*;

    #[test_case]
    fn test_mount() {
        let mount_path = "/";
        let path = "/dev/zero";
        let mut vfs = Vfs::new();

        assert_eq!(Errno::ENOENT, vfs.find_vnode(&path).err().unwrap());

        let node = Arc::new(Mutex::new(DevFs::new().into_root()));
        vfs.mount(&mount_path, node).unwrap();

        assert!(vfs.find_vnode(&path).is_ok());
    }

    #[test_case]
    fn test_mount_enoent() {
        let mut vfs = Vfs::new();
        let node = Arc::new(Mutex::new(DevFs::new().into_root()));
        assert_eq!(
            Errno::ENOENT,
            vfs.mount(&"/path/does/not/exist", node).err().unwrap()
        );
    }

    #[test_case]
    fn test_mount_already_mounted() {
        let mut vfs = Vfs::new();
        let node = Arc::new(Mutex::new(DevFs::new().into_root()));
        vfs.mount(&"/", node.clone()).unwrap();

        assert_eq!(Errno::EBUSY, vfs.mount(&"/", node).err().unwrap());
    }
}
