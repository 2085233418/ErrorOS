//! 文件系统模块

pub mod file;
pub mod inode;
pub mod fd_table;
pub mod stdio;
pub mod ramfs;
pub mod manager;
pub mod inspector;      // 真实文件系统状态查询模块

pub use file::{File, FileError, FileType, FileMetadata, SeekFrom};
pub use inode::{Inode, MemInode, InodeHandle, permissions};
pub use fd_table::{FileDescriptor, FileDescriptorTable, STDIN, STDOUT, STDERR};
pub use stdio::{Stdin, Stdout, Stderr};
pub use ramfs::{RamFS, RamInode, RamFile, DirEntry};
pub use manager::{RAMFS, FD_TABLE, init};
