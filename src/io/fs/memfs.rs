use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use kstd::sync::RwLock;

use crate::io::fs::perm::Permission;
use crate::io::fs::{CreateNodeType, Fs, IDir, IFile, INode, INodeBase, INodeNum, Stat};
use kstd::io::{Error, Result};

pub struct MemFs {
    #[allow(dead_code)] // TODO: inner is read by tests, but remove it anyways
    inner: InnerHandle,
    root: INode,
}

type InnerHandle = Rc<RwLock<Inner>>;

struct Inner {
    nodes: BTreeMap<INodeNum, INode>,
    inode_counter: AtomicU64,
}

impl Inner {
    fn get_unused_inode_num(&self) -> INodeNum {
        self.inode_counter.fetch_add(1, Ordering::SeqCst).into()
    }
}

impl MemFs {
    pub fn new(root_node_name: String) -> Self {
        let inner = Rc::new(RwLock::new(Inner {
            nodes: BTreeMap::new(),
            inode_counter: AtomicU64::new(1),
        }));

        let root_inode_num = 0_u64.into();
        let root_dir = MemDir::new(inner.clone(), root_node_name, root_inode_num);
        let root = INode::new_dir(root_dir);
        inner.write().nodes.insert(root_inode_num, root.clone());

        Self { inner, root }
    }
}

impl Fs for MemFs {
    fn root_inode(&self) -> INode {
        self.root.clone()
    }
}

struct MemNodeBase {
    fs: Rc<RwLock<Inner>>,
    stat: Stat,
    name: String,
}

impl MemNodeBase {
    fn new(fs: InnerHandle, name: String, stat: Stat) -> Self {
        Self { fs, name, stat }
    }
}

impl INodeBase for MemNodeBase {
    fn num(&self) -> INodeNum {
        self.stat.inode
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

pub struct MemFile {
    base: MemNodeBase,
    data: Vec<u8>,
}

impl MemFile {
    fn new(fs: InnerHandle, name: String, inode_num: INodeNum, data: Vec<u8>) -> Self {
        Self {
            base: MemNodeBase::new(
                fs,
                name,
                Stat {
                    inode: inode_num,
                    size: data.len() as u64,
                    ..Default::default()
                },
            ),
            data,
        }
    }
}

impl INodeBase for MemFile {
    fn num(&self) -> INodeNum {
        self.base.num()
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        self.base.stat()
    }
}

impl IFile for MemFile {
    fn size(&self) -> u64 {
        self.stat().size
    }

    fn truncate(&mut self, size: u64) -> Result<()> {
        let new_size = TryInto::<usize>::try_into(size).unwrap(); // u64 -> usize is valid on x86_64
        self.data.resize(new_size, 0);
        self.base.stat.size = self.data.len() as u64;
        Ok(())
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();
        let length = buffer.len();
        if offset as usize + length > self.data.len() {
            return Err(Error::InvalidOffset);
        }
        buffer.copy_from_slice(&self.data[offset as usize..offset as usize + length]);
        Ok(length)
    }

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        let buffer = buf.as_ref();
        let length = buffer.len();
        if offset as usize + length > self.data.len() {
            return Err(Error::InvalidOffset);
        }
        self.data[offset as usize..offset as usize + length].copy_from_slice(buffer);
        Ok(length)
    }
}

pub struct MemDir {
    base: MemNodeBase,
    children: Vec<INodeNum>,
}

impl MemDir {
    fn new(fs: InnerHandle, name: String, inode_num: INodeNum) -> Self {
        Self {
            base: MemNodeBase::new(
                fs,
                name,
                Stat {
                    inode: inode_num,
                    ..Default::default()
                },
            ),
            children: vec![],
        }
    }
}

impl INodeBase for MemDir {
    fn num(&self) -> INodeNum {
        self.base.num()
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        self.base.stat()
    }
}

impl IDir for MemDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode> {
        let needle = name.as_ref();
        let guard = self.base.fs.read();
        match self
            .children
            .iter()
            .filter_map(|n| guard.nodes.get(n))
            .find(|n| n.name() == needle)
        {
            None => Err(Error::NotFound),
            Some(n) => Ok(n.clone()),
        }
    }

    fn create(
        &mut self,
        name: &dyn AsRef<str>,
        typ: CreateNodeType,
        _permission: Permission,
    ) -> Result<INode> {
        let name = name.as_ref().to_string();
        let inode_num = self.base.fs.read().get_unused_inode_num();
        let inode = match typ {
            CreateNodeType::File => {
                let f = MemFile::new(self.base.fs.clone(), name, inode_num, vec![]);
                INode::new_file(f)
            }
            CreateNodeType::Dir => {
                let d = MemDir::new(self.base.fs.clone(), name, inode_num);
                INode::new_dir(d)
            }
        };
        self.mount(inode.clone())?;
        Ok(inode)
    }

    fn children(&self) -> Result<Vec<INode>> {
        let guard = self.base.fs.read();
        Ok(self
            .children
            .iter()
            .filter_map(|n| guard.nodes.get(n))
            .cloned()
            .collect())
    }

    fn mount(&mut self, node: INode) -> Result<()> {
        let inode_num = node.num();
        self.base.fs.write().nodes.insert(inode_num, node);
        self.children.push(inode_num);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test_case]
    fn test_file_truncate() {
        macro_rules! assert_size {
            ($f:expr, $sz:expr) => {
                assert_eq!($sz, $f.size(), "f.size()");
                assert_eq!($sz, $f.data.len(), "f.data.len()");
                assert_eq!($sz, $f.base.stat.size, "f.base.stat.size");
            };
        }

        let fs = MemFs::new("mem".into());
        let mut f = MemFile::new(fs.inner, "file.txt".into(), 0_u64.into(), vec![]);
        assert_size!(f, 0);

        f.truncate(1).unwrap();
        assert_size!(f, 1);

        f.truncate(0).unwrap();
        assert_size!(f, 0);

        f.truncate(1000).unwrap();
        assert_size!(f, 1000);
    }

    #[test_case]
    fn test_file_write_at() {
        let fs = MemFs::new("mem".into());
        let mut f = MemFile::new(fs.inner, "file.txt".into(), 0_u64.into(), vec![]);
        let data = "Hello, World!";
        f.truncate(data.len() as u64).unwrap();
        f.write_at(0, &data.as_bytes()).unwrap();
        assert_eq!(data.len(), f.data.len());
        assert_eq!(&data.as_bytes(), &f.data.as_slice());
    }

    #[test_case]
    fn test_file_read_at() {
        let data = Vec::from("Hello, World!".to_string());
        let fs = MemFs::new("mem".into());
        let f = MemFile::new(fs.inner, "file.txt".into(), 0_u64.into(), data.clone());
        let mut buf = vec![0_u8; data.len()];
        f.read_at(0, &mut buf).unwrap();
        assert_eq!(data, buf);
    }

    #[test_case]
    fn test_dir_lookup() {
        let fs = MemFs::new("mem".into());
        let mut d = MemDir::new(fs.inner.clone(), "/".into(), 0_u64.into());

        let f_inode_num = 1_u64.into();
        let f = MemFile::new(fs.inner.clone(), "file.txt".into(), f_inode_num, vec![]);
        let inode = INode::new_file(f);

        // register file in dir and fs
        d.children.push(f_inode_num);
        fs.inner.write().nodes.insert(f_inode_num, inode);

        // actual testing
        assert_eq!(Err(Error::NotFound), d.lookup(&"foobar"));
        let _ = d.lookup(&"file.txt").unwrap();
    }

    #[test_case]
    fn test_fs_create_file() {
        let fs = MemFs::new("mem".into());
        let file = fs
            .root_inode()
            .as_dir()
            .expect("root must be a dir")
            .write()
            .create(&"file.txt", CreateNodeType::File, Permission::user_rwx())
            .expect("creating a file should not fail")
            .as_file()
            .expect("created inode must be a file");
        assert_eq!(0, file.read().size());

        let data = "Hello, World!";
        // first write into the created file
        {
            let mut guard = file.write();
            guard.truncate(data.len() as u64).unwrap();
            guard.write_at(0, &data).unwrap();
        }
        // then read the written data and check for correctness
        {
            let guard = file.read();
            let mut buffer = vec![0_u8; data.len()];
            guard.read_at(0, &mut buffer).unwrap();
            assert_eq!(&data.as_bytes(), &buffer.as_slice());
        }
    }
}
