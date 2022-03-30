use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

use bitflags::bitflags;
use spin::Mutex;

use crate::driver::ide::channel::IDEChannel;
use crate::driver::ide::drive::IDEDrive;
use crate::driver::pci::classes::{InterruptPin, MassStorageSubClass, PCIDeviceClass};
use crate::driver::pci::header::PCIStandardHeaderDevice;

pub mod channel;
pub mod drive;

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

impl From<Command> for u8 {
    fn from(c: Command) -> u8 {
        c as u8
    }
}

pub struct IDEController {
    primary: Arc<Mutex<IDEChannel>>,
    secondary: Arc<Mutex<IDEChannel>>,
    interrupt_pin: InterruptPin,
    interrupt_line: Option<u8>,
}

impl IDEController {
    pub fn drives(&self) -> Vec<IDEDrive> {
        // FIXME: don't create new devices, instead store them and hand them out refcounted
        vec![
            IDEDrive::new(self.primary.clone(), 0xA0),
            IDEDrive::new(self.primary.clone(), 0xB0),
            IDEDrive::new(self.secondary.clone(), 0xA0),
            IDEDrive::new(self.secondary.clone(), 0xB0),
        ]
    }
}

impl From<PCIStandardHeaderDevice> for IDEController {
    fn from(device: PCIStandardHeaderDevice) -> Self {
        let class = device.class();
        match class {
            PCIDeviceClass::MassStorageController(sub) => match sub {
                MassStorageSubClass::IDEController => {}
                _ => panic!("mass storage controller is not an IDE controller"),
            },
            _ => panic!("pci device is not a mass storage controller"),
        }

        /*
        TODO: the following two TODOs refer to
        https://wiki.osdev.org/IDE
        where it is stated that
        "Note that BAR1 and BAR3 specify 4 ports, but only the port at
        offset 2 is used. Offsets 0, 1, and 3 should not be accessed."
         */

        let prog_if = device.prog_if();
        let (primary_ctrlbase, primary_iobase) = if is_bit_set(prog_if as u64, 0) {
            (device.bar1() as u16, device.bar0() as u16)
        } else {
            (0x3F6, 0x1F0)
        };

        let (secondary_ctrlbase, secondary_iobase) = if is_bit_set(prog_if as u64, 2) {
            (device.bar3() as u16, device.bar2() as u16)
        } else {
            (0x376, 0x170)
        };

        let bus_master_ide = device.bar4();
        let primary_master_base = bus_master_ide as u16;
        let secondary_master_base = (bus_master_ide >> 16) as u16;

        let mut primary_channels =
            IDEChannel::new(primary_ctrlbase, primary_iobase, primary_master_base);
        let mut secondary_channels =
            IDEChannel::new(secondary_ctrlbase, secondary_iobase, secondary_master_base);
        unsafe {
            // disable interrupts
            primary_channels.disable_irq();
            secondary_channels.disable_irq();
        }

        IDEController {
            primary: Arc::new(Mutex::new(primary_channels)),
            secondary: Arc::new(Mutex::new(secondary_channels)),
            interrupt_pin: device.interrupt_pin(),
            interrupt_line: device.interrupt_line(),
        }
    }
}

impl Debug for IDEController {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEController")
            .field("primary", &self.primary)
            .field("secondary", &self.secondary)
            .field("interrupt pin", &self.interrupt_pin)
            .field("interrupt line", &self.interrupt_line)
            .finish()
    }
}

fn is_bit_set(haystack: u64, needle: u8) -> bool {
    (haystack & (1 << needle)) > 0
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
