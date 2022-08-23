#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![test_runner(martim::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use martim::io::fs::{vfs, IDirHandle};
use martim::{scheduler, vfs_setup};

entry_point!(main);

#[allow(clippy::empty_loop)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    martim::init();
    martim::memory::init_heap(boot_info);
    scheduler::init();
    vfs::init();
    vfs_setup::init_vfs();

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    martim::test_panic_handler(info);
}

fn root_node() -> IDirHandle {
    // the path is not only used here, use search and replace on this file if you want to change this
    vfs::find_inode(&"/mnt/block_device0")
        .expect("root node not found at /mnt/ide1")
        .as_dir()
        .expect("mount root node is not a directory")
}

#[test_case]
fn test_filenames() {
    let root = root_node();
    let filenames_node = root
        .read()
        .lookup(&"filenames")
        .expect("not found")
        .as_dir()
        .expect("not a directory");
    let guard = filenames_node.read();
    for filename in &[
        "file1",
        "file2",
        "file_with_a_long_filename",
        "file_with_a_name_so_long_i_dont_even_know_whats_going_on_anymore",
        // 255 characters in the filename
        concat!(
            "0123456789", // 1
            "0123456789", // 2
            "0123456789", // 3
            "0123456789", // 4
            "0123456789", // 5
            "0123456789", // 6
            "0123456789", // 7
            "0123456789", // 8
            "0123456789", // 9
            "0123456789", // 10
            "0123456789", // 11
            "0123456789", // 12
            "0123456789", // 13
            "0123456789", // 14
            "0123456789", // 15
            "0123456789", // 16
            "0123456789", // 17
            "0123456789", // 18
            "0123456789", // 19
            "0123456789", // 20
            "0123456789", // 21
            "0123456789", // 22
            "0123456789", // 23
            "0123456789", // 24
            "0123456789", // 25
            "01234",
        ),
    ] {
        guard
            .lookup(filename)
            .unwrap_or_else(|_| panic!("{} not found", filename));
    }
}

fn filecontent_read_file(name: &str) -> kstd::io::Result<Vec<u8>> {
    vfs::read_file_node(&format!("/mnt/block_device0/filecontent/{}", name).as_str())
}

#[test_case]
fn test_filecontent_hello_world() {
    assert_eq!(
        "Hello, World!\n",
        String::from_utf8(filecontent_read_file("hello_world.txt").unwrap()).unwrap()
    );
}

#[test_case]
fn test_filecontent_100_bytes() {
    assert_eq!(
        &[
            0x63, 0x15, 0xd2, 0xe9, 0xdf, 0x6a, 0x63, 0x71, 0x64, 0x96, 0xde, 0xe9, 0xc9, 0x0c,
            0xaa, 0x6e, 0xbb, 0x1b, 0x38, 0x13, 0x29, 0xa9, 0x26, 0x06, 0x2c, 0x73, 0x1d, 0xdd,
            0xb2, 0x53, 0x27, 0x67, 0x88, 0x6d, 0x99, 0x0f, 0x13, 0x3d, 0x27, 0x4c, 0x2c, 0x22,
            0xaf, 0x39, 0x7e, 0xa9, 0x8b, 0x36, 0x0b, 0x75, 0x29, 0xb0, 0xa5, 0x87, 0x3a, 0xbf,
            0xfa, 0x54, 0xaa, 0xc7, 0xaf, 0x7f, 0x9a, 0x56, 0x9d, 0x13, 0x06, 0xda, 0x03, 0x47,
            0x9d, 0xc3, 0x71, 0x3d, 0xb1, 0x8a, 0x81, 0x44, 0x0f, 0x34, 0xfb, 0xc5, 0x47, 0xa5,
            0xe8, 0x01, 0xe2, 0xa9, 0xa6, 0xba, 0x70, 0x1f, 0x83, 0x87, 0x23, 0xbe, 0x8a, 0x3b,
            0x3c, 0xab
        ],
        filecontent_read_file("100_bytes.dat").unwrap().as_slice()
    );
}

#[test_case]
fn test_filecontent_1kib() {
    assert_eq!(
        &[1_u8; 1024],
        filecontent_read_file("1KiB_bytes.dat").unwrap().as_slice()
    );
}

#[test_case]
fn test_filecontent_2089_bytes() {
    assert_eq!(
        &[0x78_u8; 2089],
        filecontent_read_file("2089_bytes.dat").unwrap().as_slice()
    );
}

#[test_case]
fn test_filecontent_4kib() {
    assert_eq!(
        &[4_u8; 4 * 1024],
        filecontent_read_file("4KiB_bytes.dat").unwrap().as_slice()
    );
}

#[test_case]
fn test_filecontent_64kib() {
    let file = vfs::find_inode(&"/mnt/block_device0/filecontent/64KiB_bytes.dat")
        .expect("not found")
        .as_file()
        .expect("not a file");
    let guard = file.read();

    let mut buf = [0_u8; 4096];
    for i in 0_u64..16 {
        assert_eq!(buf.len(), guard.read_at(i * 4096, &mut buf).unwrap());
        assert_eq!(&[15_u8; 4096], &buf);
    }
}

#[test_case]
fn test_filecontent_256kib() {
    let file = vfs::find_inode(&"/mnt/block_device0/filecontent/256KiB_bytes.dat")
        .expect("not found")
        .as_file()
        .expect("not a file");
    let guard = file.read();

    let mut buf = [0_u8; 4096];
    for i in 0_u64..64 {
        assert_eq!(buf.len(), guard.read_at(i * 4096, &mut buf).unwrap());
        assert_eq!(&[120_u8; 4096], &buf);
    }
}

#[test_case]
fn test_symlinks_same_level() {
    let root = root_node();
    let symlinks_node = root
        .read()
        .lookup(&"symlinks")
        .expect("not found")
        .as_dir()
        .expect("not a directory");
    macro_rules! assert_symlink_points_to {
        ($target:expr, $symlink_path:expr) => {
            assert_eq!(
                $target,
                symlinks_node
                    .read()
                    .lookup(&$symlink_path)
                    .expect("not found")
                    .as_symlink()
                    .expect("not a symlink")
                    .read()
                    .target()
                    .unwrap()
            );
        };
    }
    assert_symlink_points_to!("target_file", "symlink_file");
    assert_symlink_points_to!("target_folder", "symlink_folder");
    assert_symlink_points_to!("target_folder/cough.txt", "symlink_folder_file");
}
