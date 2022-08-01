use alloc::format;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};

use kstd::sync::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

use crate::driver::ide::channel::IDEChannel;
use crate::driver::ide::{is_bit_set, Command, Status, UDMAMode};
use kstd::io::device::block::BlockDevice;
use kstd::io::Result;

pub struct IDEDrive {
    channel: Arc<Mutex<IDEChannel>>,

    drive: u8,

    exists: bool,

    // The following block consists of the identify_sector and then values
    // that were read from it.
    identify_sector: [u16; 256],
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

    pub fn ctrlbase(&self) -> u16 {
        self.channel.lock().ctrlbase()
    }

    pub fn iobase(&self) -> u16 {
        self.channel.lock().iobase()
    }

    pub fn drive_num(&self) -> u8 {
        self.drive
    }
}

impl IDEDrive {
    fn identify(&mut self) -> bool {
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
}

impl BlockDevice for IDEDrive {
    fn block_size(&self) -> usize {
        512
    }

    fn block_count(&self) -> usize {
        TryInto::<usize>::try_into(self.sector_count).expect("too many blocks")
    }

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let mut data = [0_u16; 256];

        let lba = block;
        let sector_count = 1;

        let mut channel = self.channel.lock();
        unsafe {
            channel
                .ports
                .drive_select
                .write((0x40 + self.drive) | (((lba >> 24) & 0x0F) as u8) as u8);
            channel.ports.features.write(0);
            channel.ports.sector_count.write(sector_count);
            channel.ports.lba_lo.write(lba as u8);
            channel.ports.lba_mid.write((lba >> 8) as u8);
            channel.ports.lba_hi.write((lba >> 16) as u8);
            channel.write_command(Command::ReadSectors);
            channel.disable_irq();
            channel.wait_for_not_busy();
            without_interrupts(|| {
                channel.wait_for_ready();
                while !channel.status().contains(Status::DATA_READY) {}
                for b in &mut data {
                    *b = channel.ports.data.read();
                }
            });
            channel.poll_on_status(|status| {
                status.contains(Status::READY) && !status.contains(Status::BUSY)
            });
        }

        let target = buf.as_mut();
        let data_u8 = unsafe { data.as_slice().align_to::<u8>().1 };
        target.copy_from_slice(&data_u8[0..target.len()]);
        Ok(target.len())
    }

    fn write_block(&mut self, block: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        let buffer = buf.as_ref();
        let word_buffer = unsafe { buffer.align_to::<u16>().1 };

        let lba = block;
        let sector_count = 1;

        let mut channel = self.channel.lock();
        unsafe {
            channel
                .ports
                .drive_select
                .write((0x40 + self.drive) | (((lba >> 24) & 0x0F) as u8) as u8);
            channel.ports.features.write(0);
            channel.ports.sector_count.write(sector_count);
            channel.ports.lba_lo.write(lba as u8);
            channel.ports.lba_mid.write((lba >> 8) as u8);
            channel.ports.lba_hi.write((lba >> 16) as u8);
            channel.write_command(Command::WriteSectors);
            channel.disable_irq();
            channel.wait_for_not_busy();
            without_interrupts(|| {
                channel.wait_for_ready();
                while !channel.status().contains(Status::DATA_READY) {}
                for &w in word_buffer {
                    channel.ports.data.write(w);
                }
                channel.write_command(Command::FlushCache);
            });
            channel.poll_on_status(|status| {
                status.contains(Status::READY) && !status.contains(Status::BUSY)
            });
        }

        Ok(buffer.len())
    }
}
