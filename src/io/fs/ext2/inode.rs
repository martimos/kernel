use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use bitflags::bitflags;

use crate::io::fs::perm::Permission;
use crate::io::fs::INodeNum;
use kstd::io::cursor::Cursor;
use kstd::io::read::Read;
use kstd::io::{Error, Result};
use kstd::{read_bytes, read_le_u16, read_le_u32, read_null_terminated_string, read_u8};

#[derive(Debug)]
pub struct Ext2INode {
    pub inode_num: INodeNum,

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
    pub symlink_short_name: String,
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
    pub fn decode(source: &mut impl Read<u8>) -> Result<Self> {
        let mode = read_le_u16!(source);
        let node_type = Ext2INodeType::try_from(mode >> 12).or(Err(Error::DecodeError))?;
        let permissions = Permission::from_bits_truncate(mode & 0x0FFF);
        let uid = read_le_u16!(source);
        let lower_size = read_le_u32!(source);
        let last_access_time = read_le_u32!(source);
        let creation_time = read_le_u32!(source);
        let last_modification_time = read_le_u32!(source);
        let deletion_time = read_le_u32!(source);
        let gid = read_le_u16!(source);
        let num_hard_links = read_le_u16!(source);
        let num_disk_sectors = read_le_u32!(source);
        let flags = Ext2INodeFlags::from_bits_truncate(read_le_u32!(source));
        let os_specific_1 = read_le_u32!(source);

        let symlink_name_and_pointer_data_area = read_bytes!(source, 60);

        let mut symlink_short_name_data =
            Cursor::new(Vec::from(symlink_name_and_pointer_data_area));
        let symlink_short_name = read_null_terminated_string!(symlink_short_name_data, 60);

        let mut pointer_data = Cursor::new(Vec::from(symlink_name_and_pointer_data_area));
        let direct_pointers = [
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
            read_le_u32!(pointer_data),
        ];
        let singly_indirect_pointer = read_le_u32!(pointer_data);
        let doubly_indirect_pointer = read_le_u32!(pointer_data);
        let triply_indirect_pointer = read_le_u32!(pointer_data);
        let generation_number = read_le_u32!(source);
        let extended_attribute_block = read_le_u32!(source);
        let upper_size_or_dir_acl = read_le_u32!(source);
        let fragment_block_address = read_le_u32!(source);
        let os_specific_2 = read_bytes!(source, 12);
        Ok(Self {
            inode_num: 0_u64.into(),

            node_type,
            permissions,
            uid,
            lower_size,
            last_access_time,
            creation_time,
            last_modification_time,
            deletion_time,
            gid,
            num_hard_links,
            num_disk_sectors,
            flags,
            os_specific_1,
            symlink_short_name,
            direct_pointers,
            singly_indirect_pointer,
            doubly_indirect_pointer,
            triply_indirect_pointer,
            generation_number,
            extended_attribute_block,
            upper_size_or_dir_acl,
            fragment_block_address,
            os_specific_2,
        })
    }

    pub fn size(&self) -> u64 {
        match self.node_type {
            Ext2INodeType::Directory => self.lower_size as u64,
            Ext2INodeType::RegularFile => {
                self.lower_size as u64 | ((self.upper_size_or_dir_acl as u64) << 32)
            }
            _ => panic!("called 'size' on neither a directory nor a file"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Ext2INodeType {
    Fifo,
    CharacterDevice,
    Directory,
    BlockDevice,
    RegularFile,
    SymbolicLink,
    UnixSocket,
}

pub struct InvalidExt2INodeType;

impl TryFrom<u16> for Ext2INodeType {
    type Error = InvalidExt2INodeType;

    fn try_from(value: u16) -> core::result::Result<Self, Self::Error> {
        Ok(match value {
            0x1 => Self::Fifo,
            0x2 => Self::CharacterDevice,
            0x4 => Self::Directory,
            0x6 => Self::BlockDevice,
            0x8 => Self::RegularFile,
            0xA => Self::SymbolicLink,
            0xC => Self::UnixSocket,
            _ => return Err(InvalidExt2INodeType),
        })
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
    pub type_indicator: Ext2IDirEntryType,
    pub name: String,
}

impl Ext2DirEntry {
    pub fn decode(source: &mut impl Read<u8>) -> Result<Self> {
        let inode = read_le_u32!(source);
        let total_size = read_le_u16!(source);
        let name_length_lower = read_u8!(source);
        let type_indicator =
            Ext2IDirEntryType::try_from(read_u8!(source)).or(Err(Error::DecodeError))?;
        let mut name_data = vec![0_u8; total_size as usize - 8];
        source.read_exact(&mut name_data)?;
        let name = String::from_utf8_lossy(&name_data[0..name_length_lower as usize]).to_string();
        Ok(Self {
            inode,
            total_size,
            name_length_lower,
            type_indicator,
            name,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Ext2IDirEntryType {
    Unknown,
    RegularFile,
    Directory,
    CharacterDevice,
    BlockDevice,
    Fifo,
    Socket,
    SymbolicLink,
}

pub struct InvalidExt2IDirEntryType;

impl TryFrom<u8> for Ext2IDirEntryType {
    type Error = InvalidExt2IDirEntryType;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Unknown,
            1 => Self::RegularFile,
            2 => Self::Directory,
            3 => Self::CharacterDevice,
            4 => Self::BlockDevice,
            5 => Self::Fifo,
            6 => Self::Socket,
            7 => Self::SymbolicLink,
            _ => return Err(InvalidExt2IDirEntryType),
        })
    }
}
