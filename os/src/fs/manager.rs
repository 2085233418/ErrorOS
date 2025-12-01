//! 文件系统管理器

use super::fd_table::{FileDescriptorTable, STDIN, STDOUT, STDERR};
use super::ramfs::RamFS;
use super::stdio::{Stdin, Stdout, Stderr};
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// 全局RamFS实例
    pub static ref RAMFS: Arc<RamFS> = Arc::new(RamFS::new());

    /// 全局文件描述符表
    pub static ref FD_TABLE: Mutex<FileDescriptorTable> = {
        let stdin = Arc::new(Mutex::new(Stdin::new()));
        let stdout = Arc::new(Mutex::new(Stdout::new()));
        let stderr = Arc::new(Mutex::new(Stderr::new()));

        Mutex::new(FileDescriptorTable::with_stdio(stdin, stdout, stderr))
    };
}

/// 初始化文件系统
pub fn init() {
    // 懒加载会在第一次访问时初始化
    let _ = &*RAMFS;
    let _ = &*FD_TABLE;
    crate::println!("[FS] File system initialized");
}
