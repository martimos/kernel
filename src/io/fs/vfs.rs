use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use kstd::sync::{Mutex, Once};

use crate::io::fs::rootdir::RootDir;
#[cfg(debug_assertions)]
use crate::io::fs::INodeBase;
use crate::io::fs::{IBlockDeviceHandle, ICharacterDeviceHandle, IFileHandle, INode, Stat};
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
    CharacterDevice(ICharacterDeviceHandle),
}

/// Attempts to open the given path and return the result as a handle.
/// Symlinks can't be opened and will be dereferenced. If that's not possible, an
/// Err value will be returned.
pub fn open(p: &dyn AsRef<Path>) -> Result<OpenResult> {
    let node = find_inode(p)?;
    match node {
        INode::File(f) => Ok(OpenResult::File(f)),
        INode::Dir(_) => Err(Error::IsDir),
        INode::BlockDevice(f) => Ok(OpenResult::BlockDevice(f)),
        INode::CharacterDevice(f) => Ok(OpenResult::CharacterDevice(f)),
        INode::Symlink(symlink) => {
            let guard = symlink.read();
            let target = guard.target_path()?;
            open(&target.as_path()) // recursion takes care of symlink chains
        }
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
            INode::CharacterDevice(_) => Err(Error::InvalidArgument),
            INode::Symlink(link) => {
                let guard = link.read();
                let target = guard.target_path()?;
                self.read_file_node(&target.as_path())
            }
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
            INode::CharacterDevice(_) => {}
            INode::Dir(dir) => {
                for child in dir.read().children()?.into_iter() {
                    self.walk_node(current_depth + 1, child.clone(), f)?;
                }
            }
            INode::File(_) => {}
            INode::Symlink(_) => {} // don't follow symlinks
        }
        Ok(())
    }

    fn mount(&mut self, p: &dyn AsRef<Path>, node: INode) -> Result<()> {
        let target_node = match self.find_inode(p) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        let dir = match target_node.clone() {
            INode::File(_) => return Err(Error::IsFile),
            INode::Dir(d) => d,
            INode::BlockDevice(_) => return Err(Error::IsFile),
            INode::CharacterDevice(_) => return Err(Error::IsFile),
            INode::Symlink(link) => {
                let guard = link.read();
                let target_path = guard.target_path()?;
                let symlink_target_node =
                    self.find_inode_from(&target_path.as_path(), target_node)?;
                if !matches!(symlink_target_node, INode::Dir(_)) {
                    return Err(Error::IsFile);
                }
                symlink_target_node.as_dir().unwrap()
            }
        };
        let mut guard = dir.write();
        guard.mount(node)
    }

    fn find_inode_from(&self, p: &dyn AsRef<Path>, starting_point: INode) -> Result<INode> {
        let path = p.as_ref().to_owned(); // OwnedPaths are always canonical
        let components = path.components();

        let mut current = starting_point;
        let mut seen_root = false;
        for component in components {
            match component {
                Component::RootDir => {
                    if seen_root {
                        panic!("unexpected root dir in the middle of a path");
                    }
                    seen_root = true;
                }
                Component::CurrentDir => {} // do nothing
                Component::ParentDir => {
                    todo!("parent dir");
                }
                Component::Normal(v) => {
                    let current_dir = match current.clone() {
                        INode::File(_) => return Err(Error::NotFound),
                        INode::Dir(d) => d,
                        INode::BlockDevice(_) => return Err(Error::NotFound),
                        INode::CharacterDevice(_) => return Err(Error::NotFound),
                        INode::Symlink(link) => {
                            let guard = link.read();
                            let target_path = guard.target_path()?;
                            debug!("symlink {} -> {:?}", current.name(), target_path);
                            let target_node =
                                self.find_inode_from(&target_path.as_path(), current)?;
                            if !matches!(target_node, INode::Dir(_)) {
                                return Err(Error::NotFound);
                            }
                            current = target_node;
                            continue; // try again with the resolved symlink as current node
                        }
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

        self.find_inode_from(p, self.root.clone())
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
            .as_dir()
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
