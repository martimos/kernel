use crate::io::fs::inode::Inode;
use alloc::{boxed::Box, rc::Rc};
use spin::Mutex;

/// A node containing an Inode and its parent.
/// Used for constructing an absolute path.
pub struct Upnode {
    pub parent: Option<Rc<Mutex<Upnode>>>,
    pub inode: Rc<Mutex<Box<dyn Inode>>>,
}

impl Upnode {
    pub fn from(parent: Rc<Mutex<Upnode>>, inode: Rc<Mutex<Box<dyn Inode>>>) -> Upnode {
        Upnode {
            parent: Some(parent),
            inode: inode.clone(),
        }
    }
}
