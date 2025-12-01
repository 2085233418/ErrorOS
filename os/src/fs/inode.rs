//! Inode抽象

use super::file::{FileType, FileMetadata};
use alloc::sync::Arc;
use spin::Mutex;

/// Inode trait - 文件元数据抽象
pub trait Inode: Send + Sync {
    fn ino(&self) -> usize;
    fn file_type(&self) -> FileType;
    fn size(&self) -> usize;
    fn mode(&self) -> u32;
}

/// 文件权限位（Unix风格）
pub mod permissions {
    pub const S_IRUSR: u32 = 0o400;
    pub const S_IWUSR: u32 = 0o200;
    pub const S_IXUSR: u32 = 0o100;

    pub const S_IRGRP: u32 = 0o040;
    pub const S_IWGRP: u32 = 0o020;
    pub const S_IXGRP: u32 = 0o010;

    pub const S_IROTH: u32 = 0o004;
    pub const S_IWOTH: u32 = 0o002;
    pub const S_IXOTH: u32 = 0o001;

    pub const S_DEFAULT_FILE: u32 = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    pub const S_DEFAULT_DIR: u32 = 0o755;
}

/// 内存中的Inode结构
#[derive(Clone)]
pub struct MemInode {
    ino: usize,
    file_type: FileType,
    mode: u32,
    size: usize,
    created: u64,
    modified: u64,
    nlinks: usize,
}

impl MemInode {
    pub fn new(ino: usize, file_type: FileType, mode: u32) -> Self {
        MemInode {
            ino,
            file_type,
            mode,
            size: 0,
            created: 0,
            modified: 0,
            nlinks: 1,
        }
    }

    pub fn new_file(ino: usize) -> Self {
        MemInode::new(ino, FileType::RegularFile, permissions::S_DEFAULT_FILE)
    }

    pub fn new_directory(ino: usize) -> Self {
        MemInode::new(ino, FileType::Directory, permissions::S_DEFAULT_DIR)
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
        self.touch();
    }

    fn touch(&mut self) {
        self.modified += 1;
    }

    pub fn inc_nlinks(&mut self) {
        self.nlinks += 1;
    }

    pub fn dec_nlinks(&mut self) {
        if self.nlinks > 0 {
            self.nlinks -= 1;
        }
    }

    pub fn nlinks(&self) -> usize {
        self.nlinks
    }

    pub fn can_delete(&self) -> bool {
        self.nlinks == 0
    }

    pub fn to_metadata(&self) -> FileMetadata {
        FileMetadata {
            file_type: self.file_type,
            size: self.size,
            mode: self.mode,
            created: self.created,
            modified: self.modified,
        }
    }

    pub fn is_readable(&self) -> bool {
        (self.mode & permissions::S_IRUSR) != 0
    }

    pub fn is_writable(&self) -> bool {
        (self.mode & permissions::S_IWUSR) != 0
    }

    pub fn is_executable(&self) -> bool {
        (self.mode & permissions::S_IXUSR) != 0
    }
}

impl Inode for MemInode {
    fn ino(&self) -> usize {
        self.ino
    }

    fn file_type(&self) -> FileType {
        self.file_type
    }

    fn size(&self) -> usize {
        self.size
    }

    fn mode(&self) -> u32 {
        self.mode
    }
}

/// Inode句柄
pub type InodeHandle = Arc<Mutex<dyn Inode>>;
