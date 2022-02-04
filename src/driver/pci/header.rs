use crate::driver::pci::device::{PCIDevice, PCIHeaderType};
use crate::driver::pci::read_config_double_word;
use core::ops::Deref;

pub struct PCIStandardHeaderDevice {
    inner: PCIDevice,
}

impl Deref for PCIStandardHeaderDevice {
    type Target = PCIDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PCIStandardHeaderDevice {
    const OFFSET_BAR0: u8 = 0x10;
    const OFFSET_BAR1: u8 = 0x14;
    const OFFSET_BAR2: u8 = 0x18;
    const OFFSET_BAR3: u8 = 0x1C;
    const OFFSET_BAR4: u8 = 0x20;
    const OFFSET_BAR5: u8 = 0x24;

    pub fn new(inner: PCIDevice) -> Self {
        if inner.header_type() != PCIHeaderType::Standard {
            panic!("pci device does not have a standard header type");
        }
        PCIStandardHeaderDevice { inner }
    }

    pub fn bar0(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR0)
    }

    pub fn bar1(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR1)
    }

    pub fn bar2(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR2)
    }

    pub fn bar3(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR3)
    }

    pub fn bar4(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR4)
    }

    pub fn bar5(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR5)
    }

    fn read_bar(&self, bar_offset: u8) -> u32 {
        unsafe {
            read_config_double_word(
                self.inner.bus(),
                self.inner.slot(),
                self.inner.function(),
                bar_offset,
            )
        }
    }
}
