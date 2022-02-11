use crate::driver::ide::channel::IDEChannel;
use crate::driver::ide::{is_bit_set, Command, Status, UDMAMode};
use crate::serial_println;
use alloc::format;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

pub struct IDEDrive {
    channel: Arc<Mutex<IDEChannel>>,

    drive: u8,

    exists: bool,
    initial_read: [u16; 256],
    supported_udma_modes: UDMAMode,
    active_udma_mode: UDMAMode,
}

impl Debug for IDEDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEDrive")
            .field("channel", &self.channel.lock())
            .field("drive", &format!("{:#X}", self.drive))
            .field("exists", &self.exists)
            .finish()
    }
}

impl IDEDrive {
    pub fn new(channel: Arc<Mutex<IDEChannel>>, drive: u8) -> Self {
        let mut drive = IDEDrive {
            channel,
            drive,
            exists: false,
            initial_read: [0; 256],
            supported_udma_modes: UDMAMode::empty(),
            active_udma_mode: UDMAMode::empty(),
        };
        drive.exists = drive.identify();
        drive
    }
}

impl IDEDrive {
    /// Tells whether or not this drive exists.
    /// If it doesn't, every operation on this drive
    /// will panic.
    pub fn exists(&self) -> bool {
        self.exists
    }

    /// Panics if this drive doesn't exist.
    fn ensure_exists(&self) {
        if !self.exists() {
            panic!("drive does not exist, check with IDEDrive::exists() before proceeding");
        }
    }
}

impl IDEDrive {
    pub fn identify(&mut self) -> bool {
        let mut channel = self.channel.lock();
        unsafe {
            channel.ports.drive_select.write(self.drive);

            channel.ports.lba_lo.write(0);
            channel.ports.lba_mid.write(0);
            channel.ports.lba_hi.write(0);

            channel.write_command(Command::Identify);
            let status = channel.status();
            if status.bits == 0 {
                // serial_println!("drive does not exist");
                return false;
            }

            while channel.status().contains(Status::BUSY) {
                // hope that this doesn't become optimized
            }
            if channel.ports.lba_mid.read() != 0 || channel.ports.lba_hi.read() != 0 {
                // serial_println!("drive is not ATA");
                return false;
            }
            loop {
                let status = channel.status();
                if status.contains(Status::ERROR) {
                    panic!("error during IDENTIFY");
                }
                if status.contains(Status::DATA_READY) {
                    break;
                }
            }

            channel.wait_for_not_busy();
            drop(channel);
            without_interrupts(|| {
                let mut channel = self.channel.lock();
                channel.wait_for_ready();
                channel.ports.command.write(Command::ReadSectors.into());

                for i in 0..self.initial_read.len() {
                    self.initial_read[i] = channel.ports.data.read();
                }
            });

            let udma_indicator = self.initial_read[88];
            self.active_udma_mode = UDMAMode::from_bits_truncate((udma_indicator >> 8) as u8);
            self.supported_udma_modes = UDMAMode::from_bits_truncate(udma_indicator as u8);
        }
        true
    }

    pub fn is_lba48_supported(&self) -> bool {
        is_bit_set(self.initial_read[83] as u64, 10)
    }

    pub fn supported_udma_modes(&self) -> UDMAMode {
        self.supported_udma_modes
    }

    pub fn active_udma_mode(&self) -> UDMAMode {
        self.active_udma_mode
    }
}
