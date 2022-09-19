use crate::driver::Peripherals;
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
    setup_vfs_base_structuce();
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
                INode::Symlink(link) => format!("-> {}", link.read().target().unwrap()),
            }
        );
    }) {
        error!("error while walking tree: {}", e);
    }

    info!("vfs initialized");
}

fn setup_vfs_base_structuce() {
    let devfs = DevFs::new("dev".to_string());
    vfs::mount(&"/", devfs.root_inode()).unwrap();

    let mntfs = MemFs::new("mnt".to_string());
    vfs::mount(&"/", mntfs.root_inode()).unwrap();
}

fn mount_ide_drive_files() {
    let drives = Peripherals::ide_drives();

    let mut i = 0;
    for &drive in drives {
        let display_string = format!("{}", drive);
        let block_device_file = BlockDeviceFile::new(drive, 0_u64.into(), format!("ide{}", i));
        i += 1;
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
        .as_dir()
        .expect("/dev should be a directory")
        .read()
        .children()
        .unwrap()
        .into_iter()
        .filter_map(|node| node.as_block_device_file())
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
