#[repr(C, packed)]
pub struct BPB {
    // documentation copied from the osdev wiki
    /// The first three bytes EB 3C 90 disassemble to JMP SHORT 3C NOP.
    /// (The 3C value may be different.) The reason for this is to jump over
    /// the disk format information (the BPB and EBPB). Since the first sector
    /// of the disk is loaded into ram at location 0x0000:0x7c00 and executed,
    /// without this jump, the processor would attempt to execute data that isn't
    /// code. Even for non-bootable volumes, code matching this pattern (or using
    /// the E9 jump opcode) is required to be present by both Windows and OS X.
    /// To fulfil this requirement, an infinite loop can be placed here with
    /// the bytes EB FE 90.
    jsn: [u8; 3],
    /// OEM identifier. The first 8 Bytes (3 - 10) is the version of DOS
    /// being used. The next eight Bytes 29 3A 63 7E 2D 49 48 and 43
    /// read out the name of the version. The official FAT Specification
    /// from Microsoft says that this field is really meaningless and is
    /// ignored by MS FAT Drivers, however it does recommend the value
    /// "MSWIN4.1" as some 3rd party drivers supposedly check it and
    /// expect it to have that value. Older versions of dos also report
    /// MSDOS5.1, linux-formatted floppy will likely to carry "mkdosfs"
    /// here, and FreeDOS formatted disks have been observed to have "FRDOS5.1"
    /// here. If the string is less than 8 bytes, it is padded with spaces.
    oem_id: [u8; 8],
    /// The number of Bytes per sector.
    bytes_per_sector: u16,
    /// Number of sectors per cluster.
    sectors_per_cluster: u8,
    /// Number of reserved sectors. The boot record sectors are included in this value.
    num_reserved_sectors: u16,
    /// Number of File Allocation Tables (FAT's) on the storage media.
    num_fat: u8,
    /// Number of root directory entries (must be set so that
    /// the root directory occupies entire sectors).
    num_root_dir_entries: u16,
    /// The total sectors in the logical volume. If this value is 0, it means
    /// there are more than 65535 sectors in the volume, and the
    /// actual count is stored in the Large Sector Count entry at 0x20.
    num_total_sectors: u16,
    /// This Byte indicates the media descriptor type.
    media_descriptor_type: u8,
    /// Number of sectors per FAT. FAT12/FAT16 only.
    sectors_per_fat: u16,
    /// Number of sectors per track.
    sectors_per_track: u16,
    /// Number of heads or sides on the storage media.
    num_heads: u16,
    /// Number of hidden sectors. (i.e. the LBA of the beginning of the partition.)
    num_hidden_sectors: u32,
    /// Large sector count. This field is set if there are more than 65535
    /// sectors in the volume, resulting in a value which does
    /// not fit in the Number of Sectors entry at 0x13.
    large_sector_count: u32,
}
