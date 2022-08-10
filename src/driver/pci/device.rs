use alloc::rc::Rc;

use bitflags::bitflags;
use kstd::sync::RwLock;

use crate::driver::pci::classes::{InterruptPin, PCIDeviceClass};
use crate::driver::pci::raw::{
    read_config_half_word, read_config_word, OFFSET_BIST, OFFSET_CLASS_SUBCLASS,
    OFFSET_HEADER_TYPE, OFFSET_INTERRUPT_LINE, OFFSET_INTERRUPT_PIN, OFFSET_PROG_IF_REVISION_ID,
    OFFSET_STATUS,
};
use crate::driver::pci::Error;
use crate::error;

bitflags! {
    pub struct Status: u16 {
        const DETECTED_PARITY_ERROR = 1 << 15;
        const SIGNALED_SYSTEM_ERROR = 1 << 14;
        const RECEIVED_MASTER_ABORT = 1 << 13;
        const RECEIVED_TARGET_ABORT = 1 << 12;
        const SIGNALED_TARGET_ABORT = 1 << 11;
        const DEVSEL_TIMING = 1 << 10 | 1 << 9;
        const MASTER_DATA_PARITY_ERROR = 1 << 8 ;
        const FAST_BACK_TO_BACK_CAPABLE = 1 << 7;
        const MHZ66_CAPABLE = 1 << 5;
        const CAPABILITIES_LIST = 1 << 4;
        const INTERRUPT = 1 << 3;
    }
}

bitflags! {
    pub struct BIST: u8 {
        const BIST_CAPABLE = 1 << 7;
        const START_BIST = 1 << 6;
        const COMPLETION_CODE = (1 << 4) - 1;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PCIHeaderType {
    Standard = 0x00,
    PCI2PCIBridge = 0x01,
    CardBusBridge = 0x02,
}

impl TryFrom<u8> for PCIHeaderType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Standard,
            0x01 => Self::PCI2PCIBridge,
            0x02 => Self::CardBusBridge,
            _ => return Err(Error::UnknownHeaderType(value)),
        })
    }
}

#[derive(Clone)]
pub struct PCIDevice {
    inner: Rc<RwLock<Inner>>,
}

struct Inner {
    bus: u8,
    slot: u8,
    function: u8,
    vendor: u16,
    device: u16,
    header_type: PCIHeaderType,
    multi_function: bool,
    class: PCIDeviceClass,
    interrupt_pin: InterruptPin,
}

impl PCIDevice {
    /// Create a new pci device from the given parameters.
    ///
    /// # Safety
    ///
    /// Creating a new pci device is unsafe because this
    /// reads from memory, which could have unintended
    /// effects. Also, the caller has to ensure that this
    /// is only called once for every combination of parameters.
    pub(in crate::driver::pci) unsafe fn new(
        bus: u8,
        slot: u8,
        function: u8,
        vendor: u16,
        device: u16,
    ) -> Result<Self, Error> {
        let header_type_raw = read_config_half_word(bus, slot, function, OFFSET_HEADER_TYPE);
        let header_type = PCIHeaderType::try_from(header_type_raw & ((1 << 7) - 1))?;
        let multi_function = header_type_raw & (1 << 7) > 0;
        let class =
            PCIDeviceClass::try_from(read_config_word(bus, slot, function, OFFSET_CLASS_SUBCLASS))?;
        let interrupt_pin = InterruptPin::try_from(read_config_half_word(
            bus,
            slot,
            function,
            OFFSET_INTERRUPT_PIN,
        ))?;
        let d = PCIDevice {
            inner: Rc::new(RwLock::new(Inner {
                bus,
                slot,
                function,
                vendor,
                device,
                header_type,
                multi_function,
                class,
                interrupt_pin,
            })),
        };
        Ok(d)
    }

    pub fn class(&self) -> PCIDeviceClass {
        self.inner.read().class
    }

    pub fn prog_if(&self) -> u8 {
        let guard = self.inner.read();
        unsafe {
            (read_config_word(
                guard.bus,
                guard.slot,
                guard.function,
                OFFSET_PROG_IF_REVISION_ID,
            ) >> 8) as u8
        }
    }

    pub fn revision_id(&self) -> u8 {
        let guard = self.inner.read();
        unsafe {
            read_config_word(
                guard.bus,
                guard.slot,
                guard.function,
                OFFSET_PROG_IF_REVISION_ID,
            ) as u8
        }
    }

    pub fn status(&self) -> Status {
        let guard = self.inner.read();
        let status =
            unsafe { read_config_word(guard.bus, guard.slot, guard.function, OFFSET_STATUS) };
        Status::from_bits_truncate(status)
    }

    pub fn bist(&self) -> BIST {
        let guard = self.inner.read();
        let bist = unsafe { read_config_word(guard.bus, guard.slot, guard.function, OFFSET_BIST) };
        BIST::from_bits_truncate(bist as u8)
    }

    pub fn interrupt_line(&self) -> Option<u8> {
        let guard = self.inner.read();
        let line = unsafe {
            read_config_word(guard.bus, guard.slot, guard.function, OFFSET_INTERRUPT_LINE) as u8
        };
        match line {
            0..=15 => Some(line),
            0xFF => None,
            _ => {
                error!("unknown interrupt line: {:#X}", line);
                None
            }
        }
    }

    pub fn interrupt_pin(&self) -> InterruptPin {
        self.inner.read().interrupt_pin
    }
}

// plain getters
impl PCIDevice {
    #[inline]
    pub fn bus(&self) -> u8 {
        self.inner.read().bus
    }

    #[inline]
    pub fn slot(&self) -> u8 {
        self.inner.read().slot
    }

    #[inline]
    pub fn function(&self) -> u8 {
        self.inner.read().function
    }

    #[inline]
    pub fn vendor(&self) -> u16 {
        self.inner.read().vendor
    }

    #[inline]
    pub fn device(&self) -> u16 {
        self.inner.read().device
    }

    #[inline]
    pub fn is_multi_function(&self) -> bool {
        self.inner.read().multi_function
    }

    #[inline]
    pub fn header_type(&self) -> PCIHeaderType {
        self.inner.read().header_type
    }
}
