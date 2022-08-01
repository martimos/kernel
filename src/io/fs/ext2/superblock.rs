use alloc::string::String;
use core::fmt::Debug;

use bitflags::bitflags;

use kstd::io::read::Read;
use kstd::io::{Error, Result};
use kstd::{read_bytes, read_le_u16, read_le_u32, read_null_terminated_string, read_u8};

#[derive(Debug)]
pub struct Superblock {
    pub num_inodes: u32,
    pub num_blocks: u32,
    pub num_superuser_reserved_blocks: u32,
    pub num_unallocated_blocks: u32,
    pub num_unallocated_inodes: u32,
    pub superblock_block_number: u32,
    pub block_size: u32,
    pub fragment_size: u32,
    pub blocks_per_group: u32,
    pub fragments_per_group: u32,
    pub inodes_per_group: u32,
    pub last_mount_time: u32,
    pub last_written_time: u32,
    pub mounts_since_fsck: u16,
    pub mounts_allowed_before_fsck: u16,
    pub magic_number: u16,
    pub state: State,
    pub error_policy: ErrorPolicy,
    pub version_minor: u16,
    pub last_fsck: u32,
    pub fsck_force_interval: u32,
    pub os_id: u32,
    pub version_major: u32,
    pub uid_for_reserved_blocks: u16,
    pub gid_for_reserved_blocks: u16,
    pub extended: Option<SuperblockExtended>,
}

impl Superblock {
    pub fn decode(source: &mut impl Read<u8>) -> Result<Self> {
        let mut s = Self {
            num_inodes: read_le_u32!(source),
            num_blocks: read_le_u32!(source),
            num_superuser_reserved_blocks: read_le_u32!(source),
            num_unallocated_blocks: read_le_u32!(source),
            num_unallocated_inodes: read_le_u32!(source),
            superblock_block_number: read_le_u32!(source),
            block_size: 1024 << read_le_u32!(source),
            fragment_size: 1024 << read_le_u32!(source),
            blocks_per_group: read_le_u32!(source),
            fragments_per_group: read_le_u32!(source),
            inodes_per_group: read_le_u32!(source),
            last_mount_time: read_le_u32!(source),
            last_written_time: read_le_u32!(source),
            mounts_since_fsck: read_le_u16!(source),
            mounts_allowed_before_fsck: read_le_u16!(source),
            magic_number: read_le_u16!(source),
            state: State::from_bits_truncate(read_le_u16!(source)),
            error_policy: ErrorPolicy::from_bits_truncate(read_le_u16!(source)),
            version_minor: read_le_u16!(source),
            last_fsck: read_le_u32!(source),
            fsck_force_interval: read_le_u32!(source),
            os_id: read_le_u32!(source),
            version_major: read_le_u32!(source),
            uid_for_reserved_blocks: read_le_u16!(source),
            gid_for_reserved_blocks: read_le_u16!(source),
            extended: None,
        };
        if s.magic_number != 0xEF53 {
            return Err(Error::InvalidMagicNumber);
        }
        if s.version_major >= 1 {
            s.extended = Some(SuperblockExtended::decode(source)?);
        }
        Ok(s)
    }

    pub fn inode_size(&self) -> u16 {
        if let Some(extended) = &self.extended {
            extended.inode_size
        } else {
            128
        }
    }
}

#[derive(Debug)]
pub struct SuperblockExtended {
    pub first_non_reserved_inode: u32,
    pub inode_size: u16,
    pub this_superblock_block_group: u16,
    pub optional_features: OptionalFeatures,
    pub required_features: RequiredFeatures,
    pub write_required_features: ReadOnlyFeatures,
    pub fsid: Ext2FsId,
    pub volume_name: String,
    pub last_mount_path: String,
    pub compression: u32,
    pub num_preallocate_blocks_file: u8,
    pub num_preallocate_blocks_directory: u8,
}

impl SuperblockExtended {
    pub fn decode(source: &mut impl Read<u8>) -> Result<Self> {
        Ok(Self {
            first_non_reserved_inode: read_le_u32!(source),
            inode_size: read_le_u16!(source),
            this_superblock_block_group: read_le_u16!(source),
            optional_features: OptionalFeatures::from_bits_truncate(read_le_u32!(source)),
            required_features: RequiredFeatures::from_bits_truncate(read_le_u32!(source)),
            write_required_features: ReadOnlyFeatures::from_bits_truncate(read_le_u32!(source)),
            fsid: Ext2FsId(read_bytes!(source, 16)),
            volume_name: read_null_terminated_string!(source, 16),
            last_mount_path: read_null_terminated_string!(source, 64),
            compression: read_le_u32!(source),
            num_preallocate_blocks_file: read_u8!(source),
            num_preallocate_blocks_directory: read_u8!(source),
        })
    }
}

#[derive(Debug)]
pub struct Ext2FsId([u8; 16]);

bitflags! {
    pub struct OptionalFeatures: u32 {
        const PREALLOCATE_FOR_DIRECTORY = 0x0001;
        const AFS_SERVER_INODES_EXIST = 0x0002;
        const HAS_JOURNAL = 0x0004;
        const INODES_EXTENDED_ATTRIBUTES = 0x0008;
        const CAN_RESIZE = 0x0010;
        const DIRECTORIES_USE_HASH_INDEX = 0x0020;
    }
}

bitflags! {
    pub struct RequiredFeatures: u32 {
        const COMPRESSION_USED = 0x0001;
        const DIRECTORY_ENTRIES_HAVE_TYPE = 0x0002;
        const NEEDS_JOURNAL_REPLAY = 0x0004;
        const USES_JOURNAL_DEVICE = 0x0008;
    }
}

bitflags! {
    pub struct ReadOnlyFeatures: u32 {
        const SPARSE_SUPERBLOCK_AND_GDTS = 0x0001;
        const USE_64BIT_FILE_SIZE = 0x0002;
        const DIRS_STORED_AS_BINARY_TREE = 0x0004;
    }
}

bitflags! {
    pub struct State: u16 {
        const CLEAN = 1;
        const ERRONOUS = 2;
    }
}

bitflags! {
    pub struct ErrorPolicy: u16 {
        const IGNORE = 1;
        const REMOUNT_READ_ONLY = 2;
        const KERNEL_PANIC = 3;
    }
}
