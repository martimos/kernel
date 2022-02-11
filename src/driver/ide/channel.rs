use crate::driver::ide::{is_bit_set, Command, Error, Status, UDMAMode};
use crate::{hlt_loop, serial_print, serial_println};
use alloc::format;
use core::fmt::{Debug, Formatter};
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

#[allow(dead_code)] // a lot of fields are unused, but they exist according to spec, so we keep them
pub struct IDEChannel {
    ctrlbase: u16,
    alternate_status: PortReadOnly<u8>,
    device_control: PortWriteOnly<u8>,
    drive_address: PortReadOnly<u8>,
    iobase: u16,
    ports: ChannelsLBA28DataPorts,
    bmide: u16,
    master_ports: ChannelsLBA28DataPorts,

    initial_read: [u16; 256],
    supported_udma_modes: UDMAMode,
    active_udma_mode: UDMAMode,
}

impl IDEChannel {
    pub fn new(ctrlbase: u16, iobase: u16, bus_master_ide: u16) -> Self {
        IDEChannel {
            ctrlbase,
            alternate_status: PortReadOnly::new(ctrlbase + 0),
            device_control: PortWriteOnly::new(ctrlbase + 0),
            drive_address: PortReadOnly::new(ctrlbase + 1),
            iobase,
            ports: ChannelsLBA28DataPorts::new(iobase),
            bmide: bus_master_ide,
            master_ports: ChannelsLBA28DataPorts::new(bus_master_ide),

            initial_read: [0; 256],
            supported_udma_modes: UDMAMode::empty(),
            active_udma_mode: UDMAMode::empty(),
        }
    }

    pub fn identify(&mut self) -> bool {
        unsafe {
            self.ports.drive_select.write(0xB0);

            self.ports.lba_lo.write(0);
            self.ports.lba_mid.write(0);
            self.ports.lba_hi.write(0);

            self.write_command(Command::Identify);
            let status = self.status();
            if status.bits == 0 {
                serial_println!("drive does not exist");
                return false;
            }

            while self.status().contains(Status::BUSY) {
                // hope that this doesn't become optimized
            }
            if self.ports.lba_mid.read() != 0 || self.ports.lba_hi.read() != 0 {
                serial_println!("drive is not ATA");
                return false;
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
                self.ports.command.write(Command::ReadSectors.into());

                for i in 0..self.initial_read.len() {
                    self.initial_read[i] = self.ports.data.read();
                }
            });

            let udma_indicator = self.initial_read[88];
            self.active_udma_mode = UDMAMode::from_bits_truncate((udma_indicator >> 8) as u8);
            self.supported_udma_modes = UDMAMode::from_bits_truncate(udma_indicator as u8);
        }
        true
    }

    pub fn write_command(&mut self, cmd: Command) {
        unsafe {
            self.ports.command.write(cmd.into());
        }
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

    /// Writes the iNIEN bit to the device control port.
    pub unsafe fn disable_irq(&mut self) {
        self.device_control.write(2);
    }

    pub fn status(&mut self) -> Status {
        unsafe { Status::from_bits_truncate(self.ports.status.read()) }
    }

    pub fn error(&mut self) -> Error {
        unsafe { Error::from_bits_truncate(self.ports.error.read()) }
    }

    pub unsafe fn wait_for_ready(&mut self) {
        while !self.status().contains(Status::READY) {}
    }

    pub unsafe fn wait_for_not_busy(&mut self) {
        for _ in 0..16 {
            let _ = self.status();
        }
        while self.status().contains(Status::BUSY) {} // wait for !BUSY
    }

    pub fn foo(&mut self) {
        /*
        This is reading from the boot drive (unfortunately), but it's reading the full drive
        and prints it to serial output.
        ~Also, this doesn't terminate. If probably gets stuck in some of the polling loops.~ not anymore
         */
        unsafe {
            for lba in 0_u32.. {
                self.ports
                    .drive_select
                    .write(0xF0 | (((lba >> 24) & 0x0F) as u8) as u8);
                self.ports.features.write(0);
                self.ports.sector_count.write(1); // sector count
                self.ports.lba_lo.write(lba as u8);
                self.ports.lba_mid.write((lba >> 8) as u8);
                self.ports.lba_hi.write((lba >> 16) as u8);
                self.write_command(Command::ReadSectors); // TODO: disable the interrupt with iNIEN
                self.wait_for_not_busy();
                let mut data = [0_u16; 256];
                without_interrupts(|| {
                    self.wait_for_ready();
                    while !self.status().contains(Status::DATA_READY) {}
                    for i in 0..256 {
                        data[i] = self.ports.data.read();
                    }
                });
                data.as_slice()
                    .align_to::<u8>()
                    .1
                    .iter()
                    .map(|&b| b as char)
                    .map(|c| {
                        return if c.is_ascii() && !c.is_control() {
                            c
                        } else {
                            '_'
                        };
                    })
                    .for_each(|c| serial_print!("{}", c));
                while !self.status().contains(Status::READY) {}
            }
        }
    }
}

impl Debug for IDEChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEChannel")
            .field("iobase", &format!("{:#X}", &self.iobase))
            .field("ctrlbase", &format!("{:#X}", &self.ctrlbase))
            .field("bmide", &format!("{:#X}", &self.bmide))
            .finish()
    }
}

pub struct ChannelsLBA28DataPorts {
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

impl ChannelsLBA28DataPorts {
    pub fn new(iobase: u16) -> Self {
        Self {
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