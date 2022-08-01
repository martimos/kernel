use alloc::borrow::ToOwned;

use kstd::sync::{Mutex, Once};

use crate::info;
use crate::io::fs::rootdir::RootDir;
use crate::io::fs::{IFileHandle, INode, INodeBase, Stat};
use kstd::io::{Error, Result};
use kstd::path::components::Component;
use kstd::path::Path;

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

pub fn mount(p: &dyn AsRef<Path>, node: INode) -> Result<()> {
    info!("mounting inode '{}' at '{}'", node.name(), p.as_ref());
    get_vfs().lock().mount(p, node)
}

pub fn find_inode(p: &dyn AsRef<Path>) -> Result<INode> {
    get_vfs().lock().find_inode(p)
}

pub fn root() -> INode {
    get_vfs().lock().root.clone()
}

pub fn open(p: &dyn AsRef<Path>) -> Result<IFileHandle> {
    let node = find_inode(p)?;
    match node {
        INode::File(f) => Ok(f),
        INode::Dir(_) => Err(Error::IsDir),
    }
}

pub struct Vfs {
    root: INode,
}

impl Vfs {
    fn new() -> Self {
        Self {
            root: INode::new_dir(RootDir::new(
                "/".into(),
                Stat {
                    inode: 0_u64.into(),
                    ..Default::default()
                },
            )),
        }
    }

    fn mount(&mut self, p: &dyn AsRef<Path>, node: INode) -> Result<()> {
        let target_node = match self.find_inode(p) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        let dir = match target_node {
            INode::File(_) => return Err(Error::IsFile),
            INode::Dir(d) => d,
        };
        let mut guard = dir.write();
        guard.mount(node)
    }

    fn find_inode(&self, p: &dyn AsRef<Path>) -> Result<INode> {
        let path = p.as_ref().to_owned(); // OwnedPaths are always canonical
        let mut components = path.components();

        // check that we got an absolute path
        let first = components.next();
        if first != Some(Component::RootDir) {
            info!("path must be absolute, but was '{}'", path);
            return Err(Error::NotFound);
        } else if first.is_none() {
            info!("path can't be empty");
            return Err(Error::NotFound);
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
                    let current_dir = match current {
                        INode::File(_) => return Err(Error::NotFound),
                        INode::Dir(d) => d,
                    };
                    let next_element = current_dir.read().lookup(&v);
                    let new_current = match next_element {
                        Ok(node) => node,
                        Err(_) => return Err(Error::NotFound),
                    };
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
    use crate::io::fs::memfs::MemFs;
    use crate::io::fs::perm::Permission;
    use crate::io::fs::rootdir::RootDir;
    use crate::io::fs::{Fs, IDir, INodeType};

    use super::*;

    #[test_case]
    fn test_root_mount_and_lookup_file() {
        test_root_mount_with_node_type(INodeType::File)
    }

    #[test_case]
    fn test_root_mount_and_lookup_dir() {
        test_root_mount_with_node_type(INodeType::Dir)
    }

    fn test_root_mount_with_node_type(typ: INodeType) {
        let name = "hello";
        let fs = MemFs::new("mem".into());
        let inode = fs
            .root_inode()
            .dir()
            .unwrap()
            .write()
            .create(&name, typ, Permission::user_rwx())
            .unwrap();
        let mut r = RootDir::new("/".into(), Stat::default());
        assert_eq!(Err(Error::NotFound), r.lookup(&name));

        r.mount(inode).unwrap();
        let res = r.lookup(&name);
        assert!(res.is_ok());
        assert_eq!(name, res.unwrap().name());
    }
}
