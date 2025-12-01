//! 标准输入输出文件

use super::file::{File, FileError};
use crate::println;

/// 标准输入
pub struct Stdin;

impl Stdin {
    pub fn new() -> Self {
        Stdin
    }
}

impl File for Stdin {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, FileError> {
        // 暂不支持键盘输入
        Err(FileError::InvalidOperation)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, FileError> {
        Err(FileError::InvalidOperation)
    }
}

/// 标准输出
pub struct Stdout;

impl Stdout {
    pub fn new() -> Self {
        Stdout
    }
}

impl File for Stdout {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, FileError> {
        Err(FileError::InvalidOperation)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        if let Ok(s) = core::str::from_utf8(buf) {
            println!("{}", s);
            Ok(buf.len())
        } else {
            Err(FileError::IoError)
        }
    }
}

/// 标准错误
pub struct Stderr;

impl Stderr {
    pub fn new() -> Self {
        Stderr
    }
}

impl File for Stderr {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, FileError> {
        Err(FileError::InvalidOperation)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        if let Ok(s) = core::str::from_utf8(buf) {
            println!("{}", s);
            Ok(buf.len())
        } else {
            Err(FileError::IoError)
        }
    }
}
