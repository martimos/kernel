use crate::driver::pci::device::{MassStorageSubClass, PCIDevice, PCIDeviceClass};
use crate::driver::pci::header::PCIStandardHeaderDevice;
use crate::{serial_print, serial_println};
use bitflags::bitflags;
use core::alloc::Layout;
use core::arch::asm;
use core::ops::Deref;
use core::ptr::read_volatile;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

bitflags! {
    pub struct UDMAMode: u8 {
        // Not sure if this is correct, but I'll leave it at that (interpreted from documentation
        // on osdev.org).

        const UDMA_1 = 1 << 0;
        const UDMA_2 = 1 << 1;
        const UDMA_3 = 1 << 2;
        const UDMA_4 = 1 << 3;
        const UDMA_5 = 1 << 4;
        const UDMA_6 = 1 << 5;
        const UDMA_7 = 1 << 6;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Command {
    Nop = 0x00,
    DeviceReset = 0x08,
    ReadSectors = 0x20,
    ReadSectorsNoRetry = 0x21,
    ReadLong = 0x22,
    ReadLongNoRetry = 0x23,
    WriteSectors = 0x30,
    WriteSectorsNoRetry = 0x31,
    WriteLong = 0x32,
    WriteLongNoRetry = 0x33,
    FormatTrack = 0x50,
    ReadMultiple = 0xC4,
    WriteMultiple = 0xC5,
    FlushCache = 0xE7,
    Identify = 0xEC,
}

impl Into<u8> for Command {
    fn into(self) -> u8 {
        self as u8
    }
}

pub struct ChannelsLBA28 {
    ctrlbase: u16,
    alternate_status: PortReadOnly<u8>,
    device_control: PortWriteOnly<u8>,
    drive_address: PortReadOnly<u8>,
    iobase: u16,
    data: Port<u16>,
    error: PortReadOnly<u8>,
    features: PortWriteOnly<u8>,
    sector_count: Port<u8>,
    lba_lo: Port<u8>,
    lba_mid: Port<u8>,
    lba_hi: Port<u8>,
    drive_select: Port<u8>,
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,
}

impl ChannelsLBA28 {
    pub fn new(ctrlbase: u16, iobase: u16) -> Self {
        ChannelsLBA28 {
            ctrlbase,
            alternate_status: PortReadOnly::new(ctrlbase + 0),
            device_control: PortWriteOnly::new(ctrlbase + 0),
            drive_address: PortReadOnly::new(ctrlbase + 1),
            iobase,
            data: Port::new(iobase),
            error: PortReadOnly::new(iobase + 1),
            features: PortWriteOnly::new(iobase + 1),
            sector_count: Port::new(iobase + 2),
            lba_lo: Port::new(iobase + 3),
            lba_mid: Port::new(iobase + 4),
            lba_hi: Port::new(iobase + 5),
            drive_select: Port::new(iobase + 6),
            status: PortReadOnly::new(iobase + 7),
            command: PortWriteOnly::new(iobase + 7),
        }
    }
}

pub struct IDEController {
    inner: PCIStandardHeaderDevice,
    primary: ChannelsLBA28,
    secondary: ChannelsLBA28,
    initial_read: [u16; 256],
    supported_udma_modes: UDMAMode,
    active_udma_mode: UDMAMode,
}

impl IDEController {
    pub fn new(device: PCIStandardHeaderDevice) -> Self {
        let class = device.class();
        match class {
            PCIDeviceClass::MassStorageController(sub) => match sub {
                MassStorageSubClass::IDEController => {}
                _ => panic!("mass storage controller is not an IDE controller"),
            },
            _ => panic!("pci device is not a mass storage controller"),
        }

        let prog_if = device.prog_if();
        let (primary_ctrlbase, primary_iobase) = match is_bit_set(prog_if as u64, 0) {
            true => (device.bar1() as u16, device.bar0() as u16), // TODO: this might not be correct
            false => (0x3F6, 0x1F0),
        };
        let (secondary_ctrlbase, secondary_iobase) = match is_bit_set(prog_if as u64, 2) {
            true => (device.bar3() as u16, device.bar2() as u16), // TODO: this might not be correct
            false => (0x376, 0x170),
        };

        let mut controller = IDEController {
            inner: device,
            primary: ChannelsLBA28::new(primary_ctrlbase, primary_iobase),
            secondary: ChannelsLBA28::new(secondary_ctrlbase, secondary_iobase),
            initial_read: [0; 256],
            active_udma_mode: UDMAMode::empty(),
            supported_udma_modes: UDMAMode::empty(),
        };

        controller.identify();

        controller
    }

    fn identify(&mut self) {
        unsafe {
            self.primary.drive_select.write(0xA0);
            self.secondary.drive_select.write(0xB0);

            self.primary.lba_lo.write(0);
            self.primary.lba_mid.write(0);
            self.primary.lba_hi.write(0);

            self.primary.command.write(Command::Identify.into());
            let status = self.status();
            if status.bits == 0 {
                panic!("drive does not exist");
            }

            while self.status().contains(Status::BUSY) {
                // hope that this doesn't become optimized
            }
            if self.primary.lba_mid.read() != 0 || self.primary.lba_hi.read() != 0 {
                panic!("drive is not ATA");
            }
            loop {
                let status = self.status();
                if status.contains(Status::ERROR) {
                    panic!("error during IDENTIFY");
                }
                if status.contains(Status::DATA_READY) {
                    break;
                }
            }

            self.wait_for_not_busy();
            without_interrupts(|| {
                self.wait_for_ready();
                self.primary.command.write(Command::ReadSectors.into());

                for i in 0..self.initial_read.len() {
                    self.initial_read[i] = self.primary.data.read();
                }
            });

            let udma_indicator = self.initial_read[88];
            self.active_udma_mode = UDMAMode::from_bits_truncate((udma_indicator >> 8) as u8);
            self.supported_udma_modes = UDMAMode::from_bits_truncate(udma_indicator as u8);
        }
    }

    pub fn is_lba48_supported(&mut self) -> bool {
        is_bit_set(self.initial_read[83] as u64, 10)
    }

    pub unsafe fn wait_for_not_busy(&mut self) {
        for _ in 0..4 {
            let _ = self.status();
        }
        while self.status().contains(Status::BUSY) {} // wait for !BUSY
    }

    pub unsafe fn wait_for_ready(&mut self) {
        while !self.status().contains(Status::READY) {}
    }

    pub fn status(&mut self) -> Status {
        unsafe { Status::from_bits_truncate(self.primary.status.read()) }
    }

    pub fn error(&mut self) -> Error {
        unsafe { Error::from_bits_truncate(self.primary.error.read()) }
    }
}

fn is_bit_set(haystack: u64, needle: u8) -> bool {
    (haystack & (1 << needle)) == (1 << needle)
}

bitflags! {
    pub struct Status: u8 {
        const ERROR = 1 << 0;
        const INDEX = 1 << 1;
        const CORRECTED_DATA = 1 << 2;
        const DATA_READY = 1 << 3; // DRQ
        const OVERLAPPED_MODE_SERVICE_REQUEST = 1 << 4;
        const DRIVE_FAULT_ERROR = 1 << 5;
        const READY = 1 << 6;
        const BUSY = 1 << 7;
    }
}

bitflags! {
    pub struct Error: u8 {
        const ADDRESS_MARK_NOT_FOUND = 1 << 0;
        const TRACK_ZERO_NOT_FOUND = 1 << 1;
        const ABORTED_COMMAND = 1 << 2;
        const MEDIA_CHANGE_REQUEST = 1 << 3;
        const ID_NOT_FOUND = 1 << 4;
        const MEDIA_CHANGED = 1 << 5;
        const UNCORRECTABLE_DATA_ERROR = 1 << 6;
        const BAD_BLOCK_DETECTED = 1 << 7;
    }
}
