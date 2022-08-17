use crate::driver::ide::IDEController;
use crate::driver::pci;
use crate::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use crate::driver::pci::header::PCIStandardHeaderDevice;
use crate::io::fs::devfs::DevFs;
use crate::io::fs::ext2::Ext2Fs;
use crate::io::fs::memfs::MemFs;
use crate::io::fs::{vfs, Fs};
use crate::{debug, info, vga_println};
use alloc::format;
use alloc::string::ToString;

pub extern "C" fn init_vfs() {
    mount_devfs();
    mount_memfs();
    mount_ide_drives();

    let hello_world = vfs::find_inode(&"/dev/ide1/executables/hello_world").unwrap();
    debug!("found hello_world: {:?}", hello_world);
}

fn mount_devfs() {
    let devfs = DevFs::new("dev".to_string());
    vfs::mount(&"/", devfs.root_inode()).unwrap();
}

fn mount_memfs() {
    let memfs = MemFs::new("mem".to_string());
    vfs::mount(&"/dev", memfs.root_inode()).unwrap();
}

fn mount_ide_drives() {
    let ide_controller = pci::devices()
        .iter()
        .find(|dev| {
            dev.class() == PCIDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
        })
        .cloned()
        .map(|d| PCIStandardHeaderDevice::new(d).unwrap())
        .map(Into::<IDEController>::into)
        .expect("need an IDE controller for this to work");

    for (count, drive) in ide_controller
        .drives()
        .into_iter()
        .filter(|d| d.exists())
        .enumerate()
    {
        debug!(
            "found IDE drive at ctrlbase={:#X} iobase={:#X} drive={:#X}",
            drive.ctrlbase(),
            drive.iobase(),
            drive.drive_num(),
        );

        let display_string = format!("{}", drive);
        let ext2_result = Ext2Fs::new_with_named_root(drive, format!("ide{}", count).as_str());
        if ext2_result.is_err() {
            debug!(
                "create ext2fs failed: {} (not an ext2 file system?)",
                ext2_result.err().unwrap()
            );
            continue;
        }
        let ext2fs = ext2_result.unwrap();
        info!("mount {} at /dev/ide{} (ext2fs)", display_string, count - 1);
        vga_println!("mount /dev/ide{} as ext2 file system", count - 1);
        vfs::mount(&"/dev", ext2fs.root_inode()).expect("mount failed");
    }
}
