use crate::driver::ide::IDEController;
use crate::driver::pci;
use crate::driver::pci::classes::{MassStorageSubClass, PCIDeviceClass};
use crate::driver::pci::header::PCIStandardHeaderDevice;
use crate::io::fs::devfs::DevFs;
use crate::io::fs::device::block::BlockDeviceFile;
use crate::io::fs::device::FileBlockDevice;
use crate::io::fs::ext2::Ext2Fs;
use crate::io::fs::memfs::MemFs;
use crate::io::fs::INodeBase;
use crate::io::fs::{vfs, Fs, INode};
use crate::{error, info, serial_println};
use alloc::format;
use alloc::string::ToString;

pub extern "C" fn init_vfs() {
    mount_devfs();
    mount_memfs();
    mount_ide_drive_files();

    mount_ext2();

    if let Err(e) = vfs::walk_tree(&"/", |depth, node| {
        serial_println!(
            "{}+ {} ({})",
            "  ".repeat(depth),
            node.name(),
            match node {
                INode::File(f) => format!("file, {} bytes", f.read().size()),
                INode::Dir(_) => "dir".to_string(),
                INode::BlockDevice(_) => "block device".to_string(),
                INode::CharacterDevice(_) => "character device".to_string(),
            }
        );
    }) {
        error!("{}", e);
    }
}

fn mount_devfs() {
    let devfs = DevFs::new("dev".to_string());
    vfs::mount(&"/", devfs.root_inode()).unwrap();
}

fn mount_memfs() {
    let memfs = MemFs::new("mnt".to_string());
    vfs::mount(&"/", memfs.root_inode()).unwrap();
}

fn mount_ide_drive_files() {
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
        let display_string = format!("{}", drive);
        let block_device_file = BlockDeviceFile::new(drive, 0_u64.into(), format!("ide{}", count));
        let block_device_node = INode::new_block_device_file(block_device_file);
        info!(
            "mount {} at /dev/{}",
            display_string,
            block_device_node.name()
        );
        vfs::mount(&"/dev", block_device_node).expect("mount failed");
    }
}

fn mount_ext2() {
    // all devices are mounted at /dev, so check all block device files for
    // the ext2 magic number 0x53 0xEF at position 1080 - 1081

    vfs::find_inode(&"/dev")
        .expect("no /dev directory")
        .dir()
        .expect("/dev should be a directory")
        .read()
        .children()
        .unwrap()
        .into_iter()
        .filter_map(|node| node.block_device_file())
        .filter(|file| {
            let mut buf = [0_u8; 2];
            file.read().read_at(1080, &mut buf).unwrap();
            buf == [0x53, 0xEF]
        })
        .enumerate()
        .map(|(num, file)| {
            Ext2Fs::new_with_named_root(
                FileBlockDevice::new(file),
                format!("block_device{}", num).as_str(),
            )
        })
        .filter_map(|fs| fs.ok())
        .map(|fs| fs.root_inode())
        .for_each(|root_inode| {
            let root_name = root_inode.name();
            vfs::mount(&"/mnt", root_inode)
                .unwrap_or_else(|_| panic!("mount of {} at /mnt failed", root_name));
        });
}
