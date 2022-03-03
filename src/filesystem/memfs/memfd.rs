use crate::filesystem::file_descriptor::{FileDescriptor, Seek};
use crate::filesystem::memfs::memfile::RefcountMemFile;
use crate::filesystem::path::owned::OwnedPath;
use crate::filesystem::stat::Stat;
use crate::syscall::error::Errno;
use crate::Result;

pub struct MemFd {
    file: RefcountMemFile,
    ptr: usize,
}

impl MemFd {
    pub fn new(file: RefcountMemFile) -> Self {
        MemFd { file, ptr: 0 }
    }
}

impl FileDescriptor for MemFd {
    fn seek(&mut self, seek: Seek) -> Result<usize> {
        match seek {
            Seek::Set(v) => self.ptr = v, // seek beyond end is allowed
            Seek::Cur(v) => {
                let cur_ptr = self.ptr as isize;
                let new_ptr = cur_ptr.checked_add(v);
                self.ptr = match new_ptr {
                    None => return Err(Errno::ESPIPE),
                    Some(v) => {
                        if v.is_negative() {
                            return Err(Errno::ESPIPE);
                        }
                        v as usize
                    }
                };
            }
            Seek::End(v) => {
                let len = self.stat()?.size;
                let new_ptr = len.checked_sub(v);
                self.ptr = new_ptr.unwrap();
                self.ptr = match new_ptr {
                    None => return Err(Errno::ESPIPE),
                    Some(v) => v,
                };
            }
        };
        Ok(self.ptr)
    }

    fn read(&mut self, buffer: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buf = buffer.as_mut();
        let guard = self.file.lock();
        let slice = &guard[self.ptr..];
        let min_len = if buf.len() < slice.len() {
            buf.len()
        } else {
            slice.len()
        };
        buf[..min_len].copy_from_slice(&slice[..min_len]);
        self.ptr += min_len;
        Ok(min_len)
    }

    fn write(&mut self, buffer: &dyn AsRef<[u8]>) -> Result<usize> {
        let buf = buffer.as_ref();
        let mut guard = self.file.lock();
        let size = guard.stat().size;

        // bounds check
        let max_index = match self.ptr.checked_add(buf.len()) {
            None => return Err(Errno::ESPIPE),
            Some(v) => v,
        };
        if max_index > size {
            guard.resize(max_index); // ensure we can index up to max_index
        }

        let slice = &mut guard[self.ptr..];

        slice[..buf.len()].copy_from_slice(&buf);

        self.ptr += buf.len();
        Ok(buf.len())
    }

    fn stat(&self) -> Result<Stat> {
        Ok(self.file.lock().stat())
    }

    fn absolute_path(&self) -> OwnedPath {
        todo!()
    }
}
