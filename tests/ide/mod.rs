use martim::driver::ide::IDEController;
use martim::driver::pci::device::{MassStorageSubClass, PCIDeviceClass};
use martim::driver::pci::header::PCIStandardHeaderDevice;
use martim::driver::pci::PCI;

#[test_case]
fn test_trivial() {
    assert_eq!(1, 1);
}

#[test_case]
fn test_read_disk_img() {
    let ide_controller = PCI::devices()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .map(PCIStandardHeaderDevice::new)
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    assert_eq!(
        2,
        ide_controller
            .drives()
            .iter()
            .filter(|drive| drive.exists())
            .count()
    ); // (1) boot drive, (2) disk.img
}
