use alloc::string::{String, ToString};

use bitflags::bitflags;

use crate::io::read::Read;
use crate::io::Result;
use crate::{read_be_u16, read_be_u32, read_be_u64, read_bytes, read_u8};

bitflags! {
    pub struct TypeFlag: u8 {
        const Regular = 0;
        const Link = 1;
        const SymLink = 2;
        const CharSpecialDevice = 3;
        const BlockSpecialDevice = 4;
        const Directory = 5;
        const FIFOSpecialFile = 6;
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct HeaderBlock {
    pub name: String,
    pub mode: u64,
    pub uid: u64,
    pub gid: u64,
    pub size: u64,
    pub mtime: [u8; 12],
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
        if read_be_u32!(source) > 0 {
            panic!("size > u64.max not supported");
        }
        let size = read_be_u64!(source);
        let mtime = read_bytes!(source, 12);
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
            && self.mtime.iter().find(|&&b| b != 0).is_none()
            && self.checksum == 0
            && self.typeflag == TypeFlag::empty()
            && self.linkname.is_empty()
            && self.magic.iter().find(|&&b| b != 0).is_none()
            && self.version == 0
            && self.uname.is_empty()
            && self.gname.is_empty()
            && self.devmajor == 0
            && self.devminor == 0
            && self.prefix.is_empty()
    }
}

fn null_terminated_string<const SZ: usize>(data: [u8; SZ]) -> String {
    let nullbyte = data
        .iter()
        .position(|&p| p == 0)
        .unwrap_or_else(|| data.len());
    let string = data.split_at(nullbyte).0;
    String::from_utf8_lossy(string).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_null_terminated_string() {
        assert_eq!(
            "hello",
            null_terminated_string([b'h', b'e', b'l', b'l', b'o', 0, b'x'])
        );
    }
}
