use crate::driver::pci::{
    read_config_word, OFFSET_CLASS_SUBCLASS, OFFSET_PROG_IF_REVISION_ID, OFFSET_STATUS,
};
use bitflags::bitflags;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PCIHeaderType {
    Standard = 0x00,
    PCI2PCIBridge = 0x01,
    CardBusBridge = 0x02,
}

impl From<u8> for PCIHeaderType {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Self::Standard,
            0x01 => Self::PCI2PCIBridge,
            0x02 => Self::CardBusBridge,
            _ => panic!("unknown header type: {:#X}", v),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PCIDeviceClass {
    Unclassified,
    MassStorageController(MassStorageSubClass),
    NetworkController(NetworkSubClass),
    DisplayController,
    MultimediaController,
    MemoryController,
    Bridge(BridgeSubclass),
    SimpleCommunicationController,
    BaseSystemPeripheral,
    InputDeviceController,
    DockingStation,
    Processor,
    // SerialBusController,
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

impl From<u16> for PCIDeviceClass {
    fn from(v: u16) -> Self {
        let class = (v >> 8) as u8;
        let sub = v as u8;
        match class {
            0x00 => Self::Unclassified,
            0x01 => Self::MassStorageController(MassStorageSubClass::from(sub)),
            0x02 => Self::NetworkController(NetworkSubClass::from(sub)),
            0x03 => Self::DisplayController,
            0x04 => Self::MultimediaController,
            0x05 => Self::MemoryController,
            0x06 => Self::Bridge(BridgeSubclass::from(sub)),
            0x07 => Self::SimpleCommunicationController,
            0x08 => Self::BaseSystemPeripheral,
            0x09 => Self::InputDeviceController,
            0x0A => Self::DockingStation,
            0x0B => Self::Processor,
            _ => panic!("unknown pci device class: {:#X}", v),
        }
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

impl From<u8> for MassStorageSubClass {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Self::SCSIBusController,
            0x01 => Self::IDEController,
            0x02 => Self::FloppyDiskController,
            0x03 => Self::IPIBusController,
            0x04 => Self::RAIDController,
            0x05 => Self::ATAController,
            0x06 => Self::SerialATAController,
            0x07 => Self::SerialAttachedSCSIController,
            0x08 => Self::NonVolatileMemoryController,
            _ => Self::Other,
        }
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

impl From<u8> for NetworkSubClass {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Self::EthernetController,
            0x01 => Self::TokenRingController,
            0x02 => Self::FDDIController,
            0x03 => Self::ATMController,
            0x04 => Self::ISDNController,
            0x05 => Self::WorldFipController,
            0x06 => Self::PICMG214MultiComputingController,
            0x07 => Self::InfinibandController,
            0x08 => Self::FabricController,
            _ => Self::Other,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BridgeSubclass {
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

impl From<u8> for BridgeSubclass {
    fn from(v: u8) -> Self {
        match v {
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
            _ => Self::Other,
        }
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
}

impl PCIDevice {
    pub(crate) fn new(
        bus: u8,
        slot: u8,
        function: u8,
        vendor: u16,
        device: u16,
        header_type: PCIHeaderType,
        multi_function: bool,
    ) -> Self {
        PCIDevice {
            bus,
            slot,
            function,
            vendor,
            device,
            header_type,
            multi_function,
        }
    }

    pub fn class(&self) -> PCIDeviceClass {
        PCIDeviceClass::from(unsafe {
            read_config_word(self.bus, self.slot, self.function, OFFSET_CLASS_SUBCLASS)
        })
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
