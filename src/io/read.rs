use crate::syscall::error::Errno;

pub trait Read {
    fn read(&mut self, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize>;

    fn read_exact(&mut self, buf: &mut dyn AsMut<[u8]>) -> crate::Result<()> {
        let mut buffer = buf.as_mut();

        while !buffer.is_empty() {
            match self.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buffer;
                    buffer = &mut tmp[n..];
                }
                Err(e) => return Err(e),
            }
        }
        if buffer.is_empty() {
            Ok(())
        } else {
            Err(Errno::EIO)
        }
    }
}

#[macro_export]
macro_rules! read_bytes {
    ($source:expr, $count:expr) => {{
        let mut buf = [0_u8; $count];
        $source
            .read_exact(&mut buf)
            .or(Err(crate::syscall::error::Errno::EIO))?;
        buf
    }};
}

#[macro_export]
macro_rules! read_u8 {
    ($source:expr) => {{
        u8::from_be_bytes(read_bytes!($source, 1))
    }};
}

#[macro_export]
macro_rules! read_be_u16 {
    ($source:expr) => {{
        u16::from_be_bytes(read_bytes!($source, 2))
    }};
}

#[macro_export]
macro_rules! read_be_u32 {
    ($source:expr) => {{
        u32::from_be_bytes(read_bytes!($source, 4))
    }};
}

#[macro_export]
macro_rules! read_be_u64 {
    ($source:expr) => {{
        u64::from_be_bytes(read_bytes!($source, 8))
    }};
}
