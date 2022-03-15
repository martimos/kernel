use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

use spin::Mutex;

use crate::io::fs::perm::Permission;

pub enum Type {
    File { size: u64 },
    Directory { children: Vec<Arc<Mutex<VNode>>> },
}

pub struct VNode {
    name: String,
    permissions: Permission,
    typ: Type,
}

impl VNode {
    pub fn new_file(name: String, permissions: Permission, size: u64) -> Self {
        Self {
            name,
            permissions,
            typ: Type::File { size },
        }
    }

    pub fn new_directory(name: String, permissions: Permission, children: Vec<VNode>) -> Self {
        Self {
            name,
            permissions,
            typ: Type::Directory {
                children: children
                    .into_iter()
                    .map(|n| Arc::new(Mutex::new(n)))
                    .collect(),
            },
        }
    }
}

impl Debug for VNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VNode")
            .field("name", &self.name)
            .field("permissions", &self.permissions)
            .finish()
    }
}

impl VNode {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn permissions(&self) -> Permission {
        self.permissions
    }

    pub fn typ(&self) -> &Type {
        &self.typ
    }

    pub fn typ_mut(&mut self) -> &mut Type {
        &mut self.typ
    }
}
