//! 文件系统测试

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::fs::{RAMFS, FD_TABLE, RamFile};
use os::{serial_println, serial_print};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::string::String;

extern crate alloc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[test_case]
fn test_create_file() {
    serial_print!("test_create_file... ");

    let root = RAMFS.root();
    let file = RAMFS.create_file(root.clone(), String::from("test.txt")).unwrap();

    assert_eq!(file.lock().ino(), 2); // root是1，这个文件是2

    serial_println!("[ok]");
}

#[test_case]
fn test_write_read() {
    serial_print!("test_write_read... ");

    let root = RAMFS.root();
    let inode = RAMFS.create_file(root, String::from("data.txt")).unwrap();
    let mut file = RAMFS.open_file(inode).unwrap();

    // 写入数据
    let data = b"Hello, RamFS!";
    let n = file.write(data).unwrap();
    assert_eq!(n, data.len());

    // 重置偏移
    file.seek(os::fs::SeekFrom::Start(0)).unwrap();

    // 读取数据
    let mut buf = [0u8; 20];
    let n = file.read(&mut buf).unwrap();
    assert_eq!(n, data.len());
    assert_eq!(&buf[..n], data);

    serial_println!("[ok]");
}

#[test_case]
fn test_fd_table() {
    serial_print!("test_fd_table... ");

    let root = RAMFS.root();
    let inode = RAMFS.create_file(root, String::from("fd_test.txt")).unwrap();
    let file = RAMFS.open_file(inode).unwrap();

    let file_arc: Arc<Mutex<dyn os::fs::File>> = Arc::new(Mutex::new(file));
    let fd = FD_TABLE.lock().alloc(file_arc).unwrap();

    assert!(fd >= 3); // 0-2是标准流

    // 验证可以获取文件
    assert!(FD_TABLE.lock().get(fd).is_some());

    // 关闭文件描述符
    assert!(FD_TABLE.lock().dealloc(fd));

    // 验证已释放
    assert!(FD_TABLE.lock().get(fd).is_none());

    serial_println!("[ok]");
}

#[test_case]
fn test_create_directory() {
    serial_print!("test_create_directory... ");

    let root = RAMFS.root();
    let dir = RAMFS.create_directory(root, String::from("testdir")).unwrap();

    assert_eq!(dir.lock().file_type(), os::fs::FileType::Directory);

    serial_println!("[ok]");
}
