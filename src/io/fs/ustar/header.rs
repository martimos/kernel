use alloc::string::{String, ToString};

use bitflags::bitflags;

use crate::io::read::Read;
use crate::io::Result;
use crate::{read_be_u16, read_be_u64, read_bytes, read_u8};

bitflags! {
    pub struct TypeFlag: u8 {
        const REGULAR = 0;
        const LINK = 1;
        const SYMLINK = 2;
        const CHAR_SPECIAL_DEVICE = 3;
        const BLOCK_SPECIAL_DEVICE = 4;
        const DIRECTORY = 5;
        const FIFO_SPECIAL_FILE = 6;
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct HeaderBlock {
    pub name: String,
    pub mode: u64,
    pub uid: u64,
    pub gid: u64,
    pub size: u64,
    pub mtime: u64,
    pub checksum: u64,
    pub typeflag: TypeFlag,
    pub linkname: String,
    pub magic: [u8; 6],
    pub version: u16,
    pub uname: String,
    pub gname: String,
    pub devmajor: u64,
    pub devminor: u64,
    pub prefix: String,
}

impl HeaderBlock {
    pub fn decode(source: &mut impl Read) -> Result<Self> {
        let name_bytes = read_bytes!(source, 100);
        let name = null_terminated_string(name_bytes);
        let mode = read_be_u64!(source);
        let uid = read_be_u64!(source);
        let gid = read_be_u64!(source);

        let size_bytes = read_bytes!(source, 11);
        let _ = read_u8!(source); // skip space
        let size = oct_to_bin(size_bytes);

        let mtime_bytes = read_bytes!(source, 11);
        let _ = read_u8!(source); // skip space
        let mtime = oct_to_bin(mtime_bytes);

        let checksum = read_be_u64!(source);
        let typeflag_byte = read_u8!(source);
        let typeflag = TypeFlag::from_bits_truncate(typeflag_byte);
        let linkname_bytes = read_bytes!(source, 100);
        let linkname = null_terminated_string(linkname_bytes);
        let magic = read_bytes!(source, 6);
        let version = read_be_u16!(source);
        let uname_bytes = read_bytes!(source, 32);
        let uname = null_terminated_string(uname_bytes);
        let gname_bytes = read_bytes!(source, 32);
        let gname = null_terminated_string(gname_bytes);
        let devmajor = read_be_u64!(source);
        let devminor = read_be_u64!(source);
        let prefix_bytes = read_bytes!(source, 155);
        let prefix = null_terminated_string(prefix_bytes);
        Ok(Self {
            name,
            mode,
            uid,
            gid,
            size,
            mtime,
            checksum,
            typeflag,
            linkname,
            magic,
            version,
            uname,
            gname,
            devmajor,
            devminor,
            prefix,
        })
    }

    pub fn is_end_block(&self) -> bool {
        self.name.is_empty()
            && self.mode == 0
            && self.uid == 0
            && self.gid == 0
            && self.size == 0
            && self.mtime == 0
            && self.checksum == 0
            && self.typeflag == TypeFlag::empty()
            && self.linkname.is_empty()
            && !self.magic.iter().any(|&b| b != 0)
            && self.version == 0
            && self.uname.is_empty()
            && self.gname.is_empty()
            && self.devmajor == 0
            && self.devminor == 0
            && self.prefix.is_empty()
    }
}

fn oct_to_bin<const SZ: usize>(data: [u8; SZ]) -> u64 {
    let mut n: u64 = 0;
    for b in &data[0..data.len()] {
        n = n << 3;
        n |= (b - 0x30) as u64;
    }
    n
}

fn null_terminated_string<const SZ: usize>(data: [u8; SZ]) -> String {
    let nullbyte = data.iter().position(|&p| p == 0).unwrap_or(data.len());
    let string = data.split_at(nullbyte).0;
    String::from_utf8_lossy(string).to_string()
}

#[cfg(test)]
mod tests {
    use core::assert_eq;

    use super::*;

    #[test_case]
    fn test_null_terminated_string() {
        assert_eq!(
            "hello",
            null_terminated_string([b'h', b'e', b'l', b'l', b'o', 0, b'x'])
        );
    }

    #[test_case]
    fn test_oct_to_bin() {
        assert_eq!(
            1025,
            oct_to_bin([b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'2', b'0', b'0', b'1'])
        );
    }
}
