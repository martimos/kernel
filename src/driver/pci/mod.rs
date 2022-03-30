use alloc::vec::Vec;
use core::mem::MaybeUninit;

use spin::Once;

use device::PCIDevice;

use crate::driver::pci::device::PCIHeaderType;

pub mod classes;
pub mod device;
pub mod header;
mod raw;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    UnknownHeaderType(u8),
    UnknownPciDeviceClass(u16),
    UnknownInterruptPin(u8),
    UnknownDisplaySubClass(u8),
    UnknownSerialBusSubClass(u8),
    UnknownMassStorageSubClass(u8),
    UnknownNetworkSubClass(u8),
    UnknownBridgeSubClass(u8),

    NotStandardHeader(PCIHeaderType),
    NotPCI2PCIBridge(PCIHeaderType),
}

pub fn devices() -> &'static Devices {
    static mut DEVICES: MaybeUninit<Devices> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    ONCE.call_once(|| unsafe {
        let mut devices = Vec::new();
        for bus in 0..=255 {
            raw::iterate_bus(bus, &mut devices);
        }
        DEVICES.as_mut_ptr().write(Devices { devices })
    });

    unsafe { &*DEVICES.as_ptr() }
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
