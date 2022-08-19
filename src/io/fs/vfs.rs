use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use kstd::sync::{Mutex, Once};

use crate::io::fs::rootdir::RootDir;
use crate::io::fs::{IBlockDeviceHandle, IFileHandle, INode, INodeBase, Stat};
use crate::{debug, info};
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

/// Attempts to locate the inode referenced by the given path, and mount the given node
/// as a child of the inode.
pub fn mount(p: &dyn AsRef<Path>, node: INode) -> Result<()> {
    debug!("mounting inode '{}' in '{}'", node.name(), p.as_ref());
    get_vfs().lock().mount(p, node)
}

/// Locates the given node and attempts to read it as regular file.
/// Will return an error if the node is not a regular file.
pub fn read_file_node(p: &dyn AsRef<Path>) -> Result<Vec<u8>> {
    get_vfs().lock().read_file_node(p)
}

pub fn walk_tree<F>(p: &dyn AsRef<Path>, f: F) -> Result<()>
where
    F: Fn(usize, INode),
{
    get_vfs().lock().walk_tree(p, f)
}

pub fn find_inode(p: &dyn AsRef<Path>) -> Result<INode> {
    get_vfs().lock().find_inode(p)
}

pub fn root() -> INode {
    get_vfs().lock().root.clone()
}

pub enum OpenResult {
    File(IFileHandle),
    BlockDevice(IBlockDeviceHandle),
}

pub fn open(p: &dyn AsRef<Path>) -> Result<OpenResult> {
    let node = find_inode(p)?;
    match node {
        INode::File(f) => Ok(OpenResult::File(f)),
        INode::Dir(_) => Err(Error::IsDir),
        INode::BlockDevice(f) => Ok(OpenResult::BlockDevice(f)),
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

    fn read_file_node(&self, p: &dyn AsRef<Path>) -> Result<Vec<u8>> {
        let node = self.find_inode(p)?;
        match node {
            INode::File(f) => f.read().read_full(),
            INode::Dir(_) => Err(Error::IsDir),
            INode::BlockDevice(_) => Err(Error::InvalidArgument),
        }
    }

    fn walk_tree<F>(&self, p: &dyn AsRef<Path>, f: F) -> Result<()>
    where
        F: Fn(usize, INode),
    {
        let node = self.find_inode(p)?;
        self.walk_node(0, node, &f)
    }

    fn walk_node<F>(&self, current_depth: usize, node: INode, f: &F) -> Result<()>
    where
        F: Fn(usize, INode),
    {
        f(current_depth, node.clone());
        match node {
            INode::BlockDevice(_) => {}
            INode::Dir(dir) => {
                for child in dir.read().children()?.into_iter() {
                    self.walk_node(current_depth + 1, child.clone(), f)?;
                }
            }
            INode::File(_) => {}
        }
        Ok(())
    }

    fn mount(&mut self, p: &dyn AsRef<Path>, node: INode) -> Result<()> {
        let target_node = match self.find_inode(p) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        let dir = match target_node {
            INode::File(_) => return Err(Error::IsFile),
            INode::Dir(d) => d,
            INode::BlockDevice(_) => return Err(Error::IsFile),
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
                        INode::BlockDevice(_) => return Err(Error::NotFound),
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
    use crate::io::fs::{CreateNodeType, Fs, IDir, INodeBase};

    use super::*;

    #[test_case]
    fn test_root_mount_and_lookup_file() {
        test_root_mount_with_node_type(CreateNodeType::File)
    }

    #[test_case]
    fn test_root_mount_and_lookup_dir() {
        test_root_mount_with_node_type(CreateNodeType::Dir)
    }

    fn test_root_mount_with_node_type(typ: CreateNodeType) {
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
