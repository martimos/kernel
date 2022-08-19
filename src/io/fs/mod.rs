use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter};

use crate::io::fs::perm::Permission;
use kstd::sync::RwLock;

use kstd::io::Result;
use kstd::io::WriteAt;
use kstd::io::{Error, ReadAt};

pub mod devfs;
pub mod device;
pub mod ext2;
pub mod flags;
pub mod memfs;
pub mod perm;
pub mod rootdir;
pub mod vfs;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct INodeNum(u64);

impl Display for INodeNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<u64>> From<T> for INodeNum {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

pub trait Fs {
    fn root_inode(&self) -> INode;
}

pub trait INodeBase {
    fn num(&self) -> INodeNum;

    fn name(&self) -> String;

    fn stat(&self) -> Stat;
}

#[derive(Copy, Clone, Default)]
pub struct Stat {
    pub dev: u64,
    pub inode: INodeNum,
    pub rdev: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: u32,
    pub mtime: u32,
    pub ctime: u32,
    pub blksize: u32,
    pub blocks: u32,
}

pub type IBlockDeviceHandle = Arc<RwLock<dyn IBlockDeviceFile>>;
pub type ICharacterDeviceHandle = Arc<RwLock<dyn ICharacterDeviceFile>>;
pub type IDirHandle = Arc<RwLock<dyn IDir>>;
pub type IFileHandle = Arc<RwLock<dyn IFile>>;

#[derive(Clone)]
pub enum INode {
    BlockDevice(IBlockDeviceHandle),
    CharacterDevice(ICharacterDeviceHandle),
    Dir(IDirHandle),
    File(IFileHandle),
}

impl PartialEq for INode {
    fn eq(&self, other: &Self) -> bool {
        self.num() == other.num() && self.stat().dev == other.stat().dev
    }
}

impl INode {
    pub fn new_file<F>(f: F) -> Self
    where
        F: 'static + IFile,
    {
        Self::File(Arc::new(RwLock::new(f)))
    }

    pub fn new_block_device_file<F>(f: F) -> Self
    where
        F: 'static + IBlockDeviceFile,
    {
        Self::BlockDevice(Arc::new(RwLock::new(f)))
    }

    pub fn new_character_device_file<F>(f: F) -> Self
    where
        F: 'static + ICharacterDeviceFile,
    {
        Self::CharacterDevice(Arc::new(RwLock::new(f)))
    }

    pub fn new_dir<D>(d: D) -> Self
    where
        D: 'static + IDir,
    {
        Self::Dir(Arc::new(RwLock::new(d)))
    }

    pub fn file(&self) -> Option<IFileHandle> {
        match self {
            INode::File(f) => Some(f.clone()),
            _ => None,
        }
    }

    pub fn dir(&self) -> Option<IDirHandle> {
        match self {
            INode::Dir(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn block_device_file(&self) -> Option<IBlockDeviceHandle> {
        match self {
            INode::BlockDevice(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, INode::File(_))
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, INode::Dir(_))
    }

    pub fn is_block_device_file(&self) -> bool {
        matches!(self, INode::BlockDevice(_))
    }

    pub fn is_character_device_file(&self) -> bool {
        matches!(self, INode::CharacterDevice(_))
    }
}

impl Display for INode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Debug for INode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("INode")
            .field(
                "type",
                &match self {
                    INode::File(_) => "File",
                    INode::Dir(_) => "Dir",
                    INode::BlockDevice(_) => "BlockDevice",
                    INode::CharacterDevice(_) => "CharacterDevice",
                },
            )
            .field("inode_num", &self.num())
            .field("name", &self.name())
            .finish()
    }
}

impl INodeBase for INode {
    fn num(&self) -> INodeNum {
        match self {
            INode::File(file) => file.read().num(),
            INode::Dir(dir) => dir.read().num(),
            INode::BlockDevice(dev) => dev.read().num(),
            INode::CharacterDevice(dev) => dev.read().num(),
        }
    }

    fn name(&self) -> String {
        match self {
            INode::File(file) => file.read().name(),
            INode::Dir(dir) => dir.read().name(),
            INode::BlockDevice(dev) => dev.read().name(),
            INode::CharacterDevice(dev) => dev.read().name(),
        }
    }

    fn stat(&self) -> Stat {
        match self {
            INode::File(file) => file.read().stat(),
            INode::Dir(dir) => dir.read().stat(),
            INode::BlockDevice(dev) => dev.read().stat(),
            INode::CharacterDevice(dev) => dev.read().stat(),
        }
    }
}

pub trait IBlockDeviceFile: INodeBase {
    fn block_count(&self) -> usize;

    fn block_size(&self) -> usize;

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<()>;

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize>;
}

pub trait ICharacterDeviceFile: INodeBase {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize>;
}

pub trait IFile: INodeBase {
    fn size(&self) -> u64;

    fn truncate(&mut self, size: u64) -> Result<()>;

    fn reserve(&mut self, additional: u64) -> Result<()> {
        self.truncate(self.size() + additional)
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize>;

    fn read_full(&self) -> Result<Vec<u8>> {
        let size = TryInto::<usize>::try_into(self.size()).unwrap(); // u64 -> usize is valid on x86_64
        let mut buffer_vec = vec![0_u8; size];
        let mut buffer = buffer_vec.as_mut_slice();
        let mut offset = 0;

        while !buffer.is_empty() {
            match self.read_at(offset, &mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    offset += n as u64;
                    buffer = &mut buffer[n..];
                }
                Err(e) => return Err(e),
            }
        }
        if buffer.is_empty() {
            Ok(buffer_vec)
        } else {
            Err(Error::PrematureEndOfInput)
        }
    }
}

impl ReadAt<u8> for dyn IFile {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        self.read_at(offset, buf)
    }
}

impl WriteAt<u8> for dyn IFile {
    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        self.write_at(offset, buf)
    }
}

pub enum CreateNodeType {
    File,
    Dir,
}

pub trait IDir: INodeBase {
    /// Searches for an [`INode`] in the immediate children of this dir by name.
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode>;

    /// Creates a new [`INode`] of the given type. The operation may fail if a type
    /// is not supported.
    fn create(
        &mut self,
        name: &dyn AsRef<str>,
        typ: CreateNodeType,
        permission: Permission,
    ) -> Result<INode>;

    /// Returns a vec of [`INodes`] that are contained within this directory.
    fn children(&self) -> Result<Vec<INode>>;

    /// Adds an (possibly external) [`INode`] to the children of this dir. External
    /// means, that the given [`INode`] does not necessarily belong to the same file
    /// system or device as this node.
    fn mount(&mut self, node: INode) -> Result<()>;
}
