use crate::filesystem::perm::Permission;
use crate::filesystem::FileSystem;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use spin::Mutex;

pub enum Type {
    File { length: u64 },
    Directory { children: Vec<VNode> },
}

pub struct VNode {
    origin: Option<Rc<Mutex<Box<dyn FileSystem>>>>,

    name: String,
    permissions: Permission,
    owning_user_id: u32,  // TODO: replace with a strong type
    owning_group_id: u32, // TODO: replace with a strong type
    typ: Type,
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
    pub fn owning_user_id(&self) -> u32 {
        self.owning_user_id
    }
    pub fn owning_group_id(&self) -> u32 {
        self.owning_group_id
    }
    pub fn typ(&self) -> &Type {
        &self.typ
    }
}
