use alloc::rc::Rc;
use alloc::string::String;
use core::fmt::{Debug, Formatter};

use spin::RwLock;

use crate::io::ReadAt;
use crate::io::WriteAt;
use crate::Result;

pub mod flags;
pub mod memfs;
pub mod nullfs;
pub mod path;
pub mod perm;
pub mod ustar;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct INodeNum(u64);

impl<T: Into<u64>> From<T> for INodeNum {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

pub trait Fs {
    fn root_inode_num(&self) -> INodeNum;
    fn get_node(&self, num: INodeNum) -> Result<INode>;
}

pub trait INodeBase {
    fn num(&self) -> INodeNum;

    fn name(&self) -> String;

    fn stat(&self) -> Stat;
}

#[derive(Copy, Clone)]
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

pub type IFileHandle = Rc<RwLock<dyn IFile>>;
pub type IDirHandle = Rc<RwLock<dyn IDir>>;

#[derive(Clone)]
pub enum INode {
    File(IFileHandle),
    Dir(IDirHandle),
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
        Self::File(Rc::new(RwLock::new(f)))
    }

    pub fn new_dir<D>(d: D) -> Self
    where
        D: 'static + IDir,
    {
        Self::Dir(Rc::new(RwLock::new(d)))
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
}

impl Debug for INode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("INode")
            .field(
                "type",
                &match self {
                    INode::File(_) => "File",
                    INode::Dir(_) => "Dir",
                },
            )
            .field("inode_num", &self.num())
            .field("name", &self.name())
            .finish()
    }
}

impl INodeBase for INode {
    #[inline]
    fn num(&self) -> INodeNum {
        match self {
            INode::File(file) => file.read().num(),
            INode::Dir(dir) => dir.read().num(),
        }
    }

    #[inline]
    fn name(&self) -> String {
        match self {
            INode::File(file) => file.read().name(),
            INode::Dir(dir) => dir.read().name(),
        }
    }

    #[inline]
    fn stat(&self) -> Stat {
        match self {
            INode::File(file) => file.read().stat(),
            INode::Dir(dir) => dir.read().stat(),
        }
    }
}

pub trait IFile: INodeBase {
    fn size(&self) -> u64;

    fn truncate(&mut self, size: u64) -> Result<()>;

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize>;
}

impl ReadAt for dyn IFile {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        self.read_at(offset, buf)
    }
}

impl WriteAt for dyn IFile {
    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        self.write_at(offset, buf)
    }
}

pub enum INodeType {
    File,
    Dir,
}

pub trait IDir: INodeBase {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode>;

    fn create(&mut self, name: &dyn AsRef<str>, typ: INodeType) -> Result<INode>;

    fn link(&mut self, name: &dyn AsRef<str>, target: &dyn INodeBase) -> Result<INode>;
}
