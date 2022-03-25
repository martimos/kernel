use alloc::string::{String, ToString};
use alloc::vec;

use bitflags::bitflags;

use crate::io::fs::perm::Permission;
use crate::io::read::Read;
use crate::{read_bytes, read_le_u16, read_le_u32, read_u8, Result};

#[derive(Debug)]
pub struct Ext2INode {
    pub node_type: Ext2INodeType,
    pub permissions: Permission,
    pub uid: u16,
    pub lower_size: u32,
    pub last_access_time: u32,
    pub creation_time: u32,
    pub last_modification_time: u32,
    pub deletion_time: u32,
    pub gid: u16,
    pub num_hard_links: u16,
    pub num_disk_sectors: u32,
    pub flags: Ext2INodeFlags,
    pub os_specific_1: u32,
    pub direct_pointers: [u32; 12],
    pub singly_indirect_pointer: u32,
    pub doubly_indirect_pointer: u32,
    pub triply_indirect_pointer: u32,
    pub generation_number: u32,
    pub extended_attribute_block: u32,
    pub upper_size_or_dir_acl: u32,
    pub fragment_block_address: u32,
    pub os_specific_2: [u8; 12],
}

impl Ext2INode {
    pub fn decode(source: &mut impl Read) -> Result<Self> {
        let mode = read_le_u16!(source);
        Ok(Self {
            node_type: Ext2INodeType::from_bits_truncate(mode >> 12),
            permissions: Permission::from_bits_truncate(mode & 0x0FFF),
            uid: read_le_u16!(source),
            lower_size: read_le_u32!(source),
            last_access_time: read_le_u32!(source),
            creation_time: read_le_u32!(source),
            last_modification_time: read_le_u32!(source),
            deletion_time: read_le_u32!(source),
            gid: read_le_u16!(source),
            num_hard_links: read_le_u16!(source),
            num_disk_sectors: read_le_u32!(source),
            flags: Ext2INodeFlags::from_bits_truncate(read_le_u32!(source)),
            os_specific_1: read_le_u32!(source),
            direct_pointers: [
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
                read_le_u32!(source),
            ],
            singly_indirect_pointer: read_le_u32!(source),
            doubly_indirect_pointer: read_le_u32!(source),
            triply_indirect_pointer: read_le_u32!(source),
            generation_number: read_le_u32!(source),
            extended_attribute_block: read_le_u32!(source),
            upper_size_or_dir_acl: read_le_u32!(source),
            fragment_block_address: read_le_u32!(source),
            os_specific_2: read_bytes!(source, 12),
        })
    }
}

bitflags! {
    pub struct Ext2INodeType: u16 {
        const FIFO = 0x1;
        const CHARACTER_DEVICE = 0x2;
        const DIRECTORY = 0x4;
        const BLOCK_DEVICE = 0x6;
        const REGULAR_FILE = 0x8;
        const SYMBOLIC_LINK = 0xA;
        const UNIX_SOCKET = 0xC;
    }
}

bitflags! {
    pub struct Ext2INodeFlags: u32 {
        const SECURE_DELETION = 0x00000001;
        const KEEP_COPY_ON_DELETION = 0x00000002;
        const COMPRESSION = 0x00000004;
        const IMMEDIATELY_FLUSH_SYNC_UPDATES = 0x00000008;
        const IMMUTABLE_FILE = 0x00000010;
        const APPEND_ONLY = 0x00000020;
        const EXCLUDE_FROM_DUMP = 0x00000040;
        const DO_NOT_UPDATE_LAST_ACCESSED = 0x00000080;
        const HASH_INDEXED_DIRECTORY = 0x00010000;
        const AFS_DIRECTORY = 0x00020000;
        const JOURNAL_FILE_DATA = 0x00040000;
    }
}

#[derive(Debug)]
pub struct Ext2DirEntry {
    pub inode: u32,
    pub total_size: u16,
    pub name_length_lower: u8,
    pub type_indicator: u8,
    pub name: String,
}

impl Ext2DirEntry {
    pub fn decode(source: &mut impl Read) -> Result<Self> {
        let inode = read_le_u32!(source);
        let total_size = read_le_u16!(source);
        let name_length_lower = read_u8!(source);
        let type_indicator = read_u8!(source);
        let mut name_data = vec![0_u8; total_size as usize - 8];
        source.read_exact(&mut name_data)?;
        let null_pos = name_data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_data.len());
        let name = String::from_utf8_lossy(&name_data[0..null_pos]).to_string();
        Ok(Self {
            inode,
            total_size,
            name_length_lower,
            type_indicator,
            name,
        })
    }
}
