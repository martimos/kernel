use bitflags::bitflags;

use crate::driver::pci::Error::UnknownHeaderType;
use crate::driver::pci::{
    read_config_word, Error, OFFSET_BIST, OFFSET_CLASS_SUBCLASS, OFFSET_HEADER_TYPE,
    OFFSET_INTERRUPT_LINE, OFFSET_INTERRUPT_PIN, OFFSET_PROG_IF_REVISION_ID, OFFSET_STATUS,
};
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
            _ => return Err(UnknownHeaderType(value)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PCIDeviceClass {
    Unclassified,
    MassStorageController(MassStorageSubClass),
    NetworkController(NetworkSubClass),
    DisplayController(DisplaySubClass),
    MultimediaController,
    MemoryController,
    Bridge(BridgeSubClass),
    SimpleCommunicationController,
    BaseSystemPeripheral,
    InputDeviceController,
    DockingStation,
    Processor,
    SerialBusController(SerialBusSubClass),
    // WirelessController,
    // IntelligentController,
    // SatelliteCommunicationController,
    // EncryptionController,
    // SignalProcessingController,
    // ProcessingAccelerator,
    // NonEssentialInstrumentation,
    // CoProcessor,
    // UnassignedClass,
}

impl TryFrom<u16> for PCIDeviceClass {
    type Error = Error;

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        let class = (v >> 8) as u8;
        let sub = v as u8;
        Ok(match class {
            0x00 => Self::Unclassified,
            0x01 => Self::MassStorageController(MassStorageSubClass::try_from(sub)?),
            0x02 => Self::NetworkController(NetworkSubClass::try_from(sub)?),
            0x03 => Self::DisplayController(DisplaySubClass::try_from(sub)?),
            0x04 => Self::MultimediaController,
            0x05 => Self::MemoryController,
            0x06 => Self::Bridge(BridgeSubClass::try_from(sub)?),
            0x07 => Self::SimpleCommunicationController,
            0x08 => Self::BaseSystemPeripheral,
            0x09 => Self::InputDeviceController,
            0x0A => Self::DockingStation,
            0x0B => Self::Processor,
            0x0C => Self::SerialBusController(SerialBusSubClass::try_from(sub)?),
            _ => return Err(Error::UnknownPciDeviceClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DisplaySubClass {
    VGACompatibleController,
    XGAController,
    NoVGA3DController,
    Other,
}

impl TryFrom<u8> for DisplaySubClass {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::VGACompatibleController,
            0x01 => Self::XGAController,
            0x02 => Self::NoVGA3DController,
            0x80 => Self::Other,
            _ => return Err(Error::UnknownDisplaySubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SerialBusSubClass {
    FireWireController,
    ACCESSBusController,
    SSA,
    USBController,
    FibreChannel,
    SMBusController,
    InfiniBandController,
    IPMIInterface,
    SERCOSInterface,
    CANbusController,
    Other,
}

impl TryFrom<u8> for SerialBusSubClass {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x0 => Self::FireWireController,
            0x1 => Self::ACCESSBusController,
            0x2 => Self::SSA,
            0x3 => Self::USBController,
            0x4 => Self::FibreChannel,
            0x5 => Self::SMBusController,
            0x6 => Self::InfiniBandController,
            0x7 => Self::IPMIInterface,
            0x8 => Self::SERCOSInterface,
            0x9 => Self::CANbusController,
            0x80 => Self::Other,
            _ => return Err(Error::UnknownSerialBusSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MassStorageSubClass {
    SCSIBusController,
    IDEController,
    FloppyDiskController,
    IPIBusController,
    RAIDController,
    ATAController,
    SerialATAController,
    SerialAttachedSCSIController,
    NonVolatileMemoryController,
    Other,
}

impl TryFrom<u8> for MassStorageSubClass {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::SCSIBusController,
            0x01 => Self::IDEController,
            0x02 => Self::FloppyDiskController,
            0x03 => Self::IPIBusController,
            0x04 => Self::RAIDController,
            0x05 => Self::ATAController,
            0x06 => Self::SerialATAController,
            0x07 => Self::SerialAttachedSCSIController,
            0x08 => Self::NonVolatileMemoryController,
            0x80 => Self::Other,
            _ => return Err(Error::UnknownMassStorageSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NetworkSubClass {
    EthernetController,
    TokenRingController,
    FDDIController,
    ATMController,
    ISDNController,
    WorldFipController,
    PICMG214MultiComputingController,
    InfinibandController,
    FabricController,
    Other,
}

impl TryFrom<u8> for NetworkSubClass {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::EthernetController,
            0x01 => Self::TokenRingController,
            0x02 => Self::FDDIController,
            0x03 => Self::ATMController,
            0x04 => Self::ISDNController,
            0x05 => Self::WorldFipController,
            0x06 => Self::PICMG214MultiComputingController,
            0x07 => Self::InfinibandController,
            0x08 => Self::FabricController,
            0x80 => Self::Other,
            _ => return Err(Error::UnknownNetworkSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BridgeSubClass {
    HostBridge,
    ISABridge,
    EISABridge,
    MCABridge,
    PCI2PCIBridge,
    PCMCIABridge,
    NuBusBridge,
    CardBusBridge,
    RACEwayBridge,
    InfiniBand2PCIBridge,
    Other,
}

impl TryFrom<u8> for BridgeSubClass {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::HostBridge,
            0x01 => Self::ISABridge,
            0x02 => Self::EISABridge,
            0x03 => Self::MCABridge,
            0x04 | 0x09 => Self::PCI2PCIBridge,
            0x05 => Self::PCMCIABridge,
            0x06 => Self::NuBusBridge,
            0x07 => Self::CardBusBridge,
            0x08 => Self::RACEwayBridge,
            0x0A => Self::InfiniBand2PCIBridge,
            0x80 => Self::Other,
            _ => return Err(Error::UnknownBridgeSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InterruptPin {
    None,
    INTA,
    INTB,
    INTC,
    INTD,
}

impl TryFrom<u8> for InterruptPin {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0 => Self::None,
            1 => Self::INTA,
            2 => Self::INTB,
            3 => Self::INTC,
            4 => Self::INTD,
            _ => return Err(Error::UnknownInterruptPin(v)),
        })
    }
}

pub struct PCIDevice {
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
    pub unsafe fn new(
        bus: u8,
        slot: u8,
        function: u8,
        vendor: u16,
        device: u16,
    ) -> Result<Self, Error> {
        let header_type_raw = read_config_word(bus, slot, function, OFFSET_HEADER_TYPE) as u8;
        let header_type = PCIHeaderType::try_from(header_type_raw & ((1 << 7) - 1))?;
        let multi_function = header_type_raw & (1 << 7) > 0;
        let class =
            PCIDeviceClass::try_from(read_config_word(bus, slot, function, OFFSET_CLASS_SUBCLASS))?;
        let interrupt_pin =
            InterruptPin::try_from(
                read_config_word(bus, slot, function, OFFSET_INTERRUPT_PIN) as u8
            )?;
        let d = PCIDevice {
            bus,
            slot,
            function,
            vendor,
            device,
            header_type,
            multi_function,
            class,
            interrupt_pin,
        };
        Ok(d)
    }

    pub fn class(&self) -> PCIDeviceClass {
        self.class
    }

    pub fn prog_if(&self) -> u8 {
        unsafe {
            (read_config_word(
                self.bus,
                self.slot,
                self.function,
                OFFSET_PROG_IF_REVISION_ID,
            ) >> 8) as u8
        }
    }

    pub fn revision_id(&self) -> u8 {
        unsafe {
            read_config_word(
                self.bus,
                self.slot,
                self.function,
                OFFSET_PROG_IF_REVISION_ID,
            ) as u8
        }
    }

    pub fn status(&self) -> Status {
        let status = unsafe { read_config_word(self.bus, self.slot, self.function, OFFSET_STATUS) };
        Status::from_bits_truncate(status)
    }

    pub fn bist(&self) -> BIST {
        let bist = unsafe { read_config_word(self.bus, self.slot, self.function, OFFSET_BIST) };
        BIST::from_bits_truncate(bist as u8)
    }

    pub fn interrupt_line(&self) -> Option<u8> {
        let line = unsafe {
            read_config_word(self.bus, self.slot, self.function, OFFSET_INTERRUPT_LINE) as u8
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
        self.interrupt_pin
    }
}

// plain getters
impl PCIDevice {
    #[inline]
    pub fn bus(&self) -> u8 {
        self.bus
    }

    #[inline]
    pub fn slot(&self) -> u8 {
        self.slot
    }

    #[inline]
    pub fn function(&self) -> u8 {
        self.function
    }

    #[inline]
    pub fn vendor(&self) -> u16 {
        self.vendor
    }

    #[inline]
    pub fn device(&self) -> u16 {
        self.device
    }

    #[inline]
    pub fn is_multi_function(&self) -> bool {
        self.multi_function
    }

    #[inline]
    pub fn header_type(&self) -> PCIHeaderType {
        self.header_type
    }
}
