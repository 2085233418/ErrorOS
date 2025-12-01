//! 文件抽象

use alloc::vec::Vec;
use core::fmt;

/// 文件trait - 统一的文件操作接口
pub trait File: Send + Sync {
    /// 读取数据到缓冲区
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;

    /// 写入数据到文件
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError>;

    /// 移动文件读写位置
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, FileError> {
        Err(FileError::InvalidOperation)
    }

    /// 读取全部内容到Vec
    fn read_all(&mut self) -> Result<Vec<u8>, FileError> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 512];

        loop {
            match self.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(FileError::EndOfFile) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(buffer)
    }

    /// 写入字符串
    fn write_str(&mut self, s: &str) -> Result<usize, FileError> {
        self.write(s.as_bytes())
    }

    /// 获取文件大小
    fn size(&self) -> Result<usize, FileError> {
        Err(FileError::InvalidOperation)
    }

    /// 获取文件元数据
    fn stat(&self) -> Result<FileMetadata, FileError> {
        Err(FileError::InvalidOperation)
    }
}

/// 文件操作错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileError {
    NotFound,
    PermissionDenied,
    EndOfFile,
    InvalidOperation,
    IoError,
    AlreadyExists,
    NotDirectory,
    IsDirectory,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileError::NotFound => write!(f, "文件未找到"),
            FileError::PermissionDenied => write!(f, "权限不足"),
            FileError::EndOfFile => write!(f, "已到达文件末尾"),
            FileError::InvalidOperation => write!(f, "无效的操作"),
            FileError::IoError => write!(f, "I/O错误"),
            FileError::AlreadyExists => write!(f, "文件已存在"),
            FileError::NotDirectory => write!(f, "不是目录"),
            FileError::IsDirectory => write!(f, "是目录"),
        }
    }
}

/// Seek的起始位置
#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(usize),
    Current(isize),
    End(isize),
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    RegularFile,
    Directory,
    CharDevice,
    BlockDevice,
    Pipe,
    SymbolicLink,
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_type: FileType,
    pub size: usize,
    pub mode: u32,
    pub created: u64,
    pub modified: u64,
}

impl FileMetadata {
    pub fn new(file_type: FileType, size: usize, mode: u32) -> Self {
        FileMetadata {
            file_type,
            size,
            mode,
            created: 0,
            modified: 0,
        }
    }
}
