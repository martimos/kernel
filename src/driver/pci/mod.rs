use crate::driver::pci::device::PCIHeaderType;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use derive_more::Display;
use device::PCIDevice;

pub mod classes;
pub mod device;
pub mod header;
mod raw;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Display)]
pub enum Error {
    #[display(fmt = "unknown header type {:#x?}", "_0")]
    UnknownHeaderType(u8),
    #[display(fmt = "unknown pci device class {:#x?}", "_0")]
    UnknownPciDeviceClass(u16),
    #[display(fmt = "unknown interrupt pin {}", "_0")]
    UnknownInterruptPin(u8),
    #[display(fmt = "unknown display sub class {:#x?}", "_0")]
    UnknownDisplaySubClass(u8),
    #[display(fmt = "unknown serial bus sub class {:#x?}", "_0")]
    UnknownSerialBusSubClass(u8),
    #[display(fmt = "unknown mass storage sub class {:#x?}", "_0")]
    UnknownMassStorageSubClass(u8),
    #[display(fmt = "unknown network sub class {:#x?}", "_0")]
    UnknownNetworkSubClass(u8),
    #[display(fmt = "unknown bridge sub class {:#x?}", "_0")]
    UnknownBridgeSubClass(u8),

    #[display(fmt = "not a standard header, but a {:?}", "_0")]
    NotStandardHeader(PCIHeaderType),
    #[display(fmt = "not a pci2pci bridge, but a {:?}", "_0")]
    NotPCI2PCIBridge(PCIHeaderType),
}

impl core::error::Error for Error {}

pub fn devices() -> impl Iterator<Item = &'static PCIDevice> {
    static DEVICES: OnceCell<Devices> = OnceCell::uninit();

    DEVICES
        .get_or_init(|| {
            let mut devices = Vec::new();
            for bus in 0..=255 {
                unsafe { raw::iterate_bus(bus, &mut devices) };
            }
            Devices { devices }
        })
        .iter()
}

pub struct Devices {
    devices: Vec<PCIDevice>,
}

impl Devices {
    pub fn iter(&self) -> DevicesIter {
        DevicesIter {
            devices: self,
            index: 0,
        }
    }
}

pub struct DevicesIter<'a> {
    devices: &'a Devices,
    index: usize,
}

impl<'a> Iterator for DevicesIter<'a> {
    type Item = &'a PCIDevice;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.devices.devices.len() {
            return None;
        }
        let item = &self.devices.devices[self.index];
        self.index += 1;
        Some(item)
    }
}
