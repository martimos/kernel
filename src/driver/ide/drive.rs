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

    // The following block consists of the identify_sector and then values
    // that were read from it.
    identify_sector: [u16; 256],
    block_size: usize,
    supported_udma_modes: UDMAMode,
    active_udma_mode: UDMAMode,
    sector_count: u64,
}

impl Debug for IDEDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEDrive")
            .field("channel", &self.channel.lock())
            .field("drive", &format!("{:#X}", self.drive))
            .field("exists", &self.exists)
            .field("sector count", &self.sector_count)
            .field("udma support", &self.supported_udma_modes)
            .field("active udma", &self.active_udma_mode)
            .finish()
    }
}

impl IDEDrive {
    pub fn new(channel: Arc<Mutex<IDEChannel>>, drive: u8) -> Self {
        let mut drive = IDEDrive {
            channel,
            drive,
            exists: false,
            identify_sector: [0; 256],
            block_size: 0,
            supported_udma_modes: UDMAMode::empty(),
            active_udma_mode: UDMAMode::empty(),
            sector_count: 0,
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
            // Drop the channel lock so that it can be used again in the following closure.
            // FIXME: This is not correct, as another thread reading the other drive on this
            // channel could lock the channel right here, and the drive select on the channel could
            // make this behave incorrectly.
            drop(channel);
            without_interrupts(|| {
                let mut channel = self.channel.lock();
                channel.wait_for_ready();
                channel.ports.command.write(Command::ReadSectors.into());

                for i in 0..self.identify_sector.len() {
                    self.identify_sector[i] = channel.ports.data.read();
                }
            });

            let udma_indicator = self.identify_sector[88];
            self.active_udma_mode = UDMAMode::from_bits_truncate((udma_indicator >> 8) as u8);
            self.supported_udma_modes = UDMAMode::from_bits_truncate(udma_indicator as u8);

            if self.is_lba48_supported() {
                self.sector_count = self.identify_sector[100] as u64
                    | ((self.identify_sector[101] as u64) << 16)
                    | ((self.identify_sector[102] as u64) << 32)
                    | ((self.identify_sector[103] as u64) << 48)
            } else {
                self.sector_count =
                    self.identify_sector[60] as u64 | ((self.identify_sector[61] as u64) << 16)
            }
        }
        true
    }

    pub fn is_lba48_supported(&self) -> bool {
        is_bit_set(self.identify_sector[83] as u64, 10)
    }

    pub fn supported_udma_modes(&self) -> UDMAMode {
        self.supported_udma_modes
    }

    pub fn active_udma_mode(&self) -> UDMAMode {
        self.active_udma_mode
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }
}
