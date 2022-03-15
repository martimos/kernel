use alloc::vec;

use crate::io::fs::perm::Permission;
use crate::io::fs::vfs::vnode::VNode;

pub struct DevFs {
    root: VNode,
}

impl !Default for DevFs {}

impl DevFs {
    pub fn new() -> Self {
        Self {
            root: VNode::new_directory(
                "dev".into(),
                Permission::empty(),
                vec![
                    VNode::new_file("zero".into(), Permission::empty(), 0),
                    VNode::new_file("one".into(), Permission::empty(), 0),
                ],
            ),
        }
    }

    pub fn into_root(self) -> VNode {
        self.root
    }
}
