//! 文件描述符表

use super::file::File;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

pub type FileDescriptor = usize;

pub const STDIN: FileDescriptor = 0;
pub const STDOUT: FileDescriptor = 1;
pub const STDERR: FileDescriptor = 2;

pub struct FdEntry {
    file: Arc<Mutex<dyn File>>,
    flags: u32,
}

impl FdEntry {
    pub fn new(file: Arc<Mutex<dyn File>>) -> Self {
        FdEntry { file, flags: 0 }
    }

    pub fn file(&self) -> Arc<Mutex<dyn File>> {
        self.file.clone()
    }
}

pub struct FileDescriptorTable {
    entries: Vec<Option<FdEntry>>,
    next_fd: FileDescriptor,
}

impl FileDescriptorTable {
    pub fn new() -> Self {
        FileDescriptorTable {
            entries: Vec::new(),
            next_fd: 3,
        }
    }

    pub fn with_stdio(
        stdin: Arc<Mutex<dyn File>>,
        stdout: Arc<Mutex<dyn File>>,
        stderr: Arc<Mutex<dyn File>>,
    ) -> Self {
        let mut table = FileDescriptorTable {
            entries: Vec::with_capacity(16),
            next_fd: 3,
        };

        table.entries.push(Some(FdEntry::new(stdin)));
        table.entries.push(Some(FdEntry::new(stdout)));
        table.entries.push(Some(FdEntry::new(stderr)));

        table
    }

    pub fn alloc(&mut self, file: Arc<Mutex<dyn File>>) -> Option<FileDescriptor> {
        let entry = FdEntry::new(file);

        for (i, slot) in self.entries.iter_mut().enumerate() {
            if slot.is_none() && i >= 3 {
                *slot = Some(entry);
                self.next_fd = i + 1;
                return Some(i);
            }
        }

        let fd = self.entries.len();
        self.entries.push(Some(entry));
        self.next_fd = fd + 1;

        Some(fd)
    }

    pub fn dealloc(&mut self, fd: FileDescriptor) -> bool {
        if fd >= 3 && fd < self.entries.len() {
            if self.entries[fd].is_some() {
                self.entries[fd] = None;
                if fd < self.next_fd {
                    self.next_fd = fd;
                }
                return true;
            }
        }
        false
    }

    pub fn get(&self, fd: FileDescriptor) -> Option<Arc<Mutex<dyn File>>> {
        self.entries.get(fd)?.as_ref().map(|entry| entry.file())
    }

    pub fn is_valid(&self, fd: FileDescriptor) -> bool {
        self.get(fd).is_some()
    }

    pub fn count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }
}
