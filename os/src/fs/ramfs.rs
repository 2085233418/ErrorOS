//! 内存文件系统（RamFS）

use super::file::{File, FileError, FileType};
use super::inode::{Inode, MemInode, permissions};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// 目录项
#[derive(Clone)]
pub struct DirEntry {
    name: String,
    inode: Arc<Mutex<RamInode>>,
}

impl DirEntry {
    pub fn new(name: String, inode: Arc<Mutex<RamInode>>) -> Self {
        DirEntry { name, inode }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn inode(&self) -> Arc<Mutex<RamInode>> {
        self.inode.clone()
    }
}

/// RamFS的Inode
pub struct RamInode {
    ino: usize,
    file_type: FileType,
    mode: u32,
    size: usize,
    created: u64,
    modified: u64,
    nlinks: usize,

    // 文件数据（对于普通文件）
    data: Vec<u8>,

    // 目录项（对于目录）
    entries: BTreeMap<String, Arc<Mutex<RamInode>>>,
}

impl RamInode {
    pub fn new_file(ino: usize) -> Self {
        RamInode {
            ino,
            file_type: FileType::RegularFile,
            mode: permissions::S_DEFAULT_FILE,
            size: 0,
            created: 0,
            modified: 0,
            nlinks: 1,
            data: Vec::new(),
            entries: BTreeMap::new(),
        }
    }

    pub fn new_directory(ino: usize) -> Self {
        RamInode {
            ino,
            file_type: FileType::Directory,
            mode: permissions::S_DEFAULT_DIR,
            size: 0,
            created: 0,
            modified: 0,
            nlinks: 1,
            data: Vec::new(),
            entries: BTreeMap::new(),
        }
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, FileError> {
        if self.file_type != FileType::RegularFile {
            return Err(FileError::IsDirectory);
        }

        if offset >= self.data.len() {
            return Ok(0);
        }

        let end = core::cmp::min(offset + buf.len(), self.data.len());
        let n = end - offset;
        buf[..n].copy_from_slice(&self.data[offset..end]);
        Ok(n)
    }

    pub fn write_at(&mut self, offset: usize, buf: &[u8]) -> Result<usize, FileError> {
        if self.file_type != FileType::RegularFile {
            return Err(FileError::IsDirectory);
        }

        let end = offset + buf.len();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }

        self.data[offset..end].copy_from_slice(buf);
        self.size = self.data.len();
        self.modified += 1;
        Ok(buf.len())
    }

    pub fn truncate(&mut self, size: usize) -> Result<(), FileError> {
        if self.file_type != FileType::RegularFile {
            return Err(FileError::IsDirectory);
        }

        self.data.resize(size, 0);
        self.size = size;
        self.modified += 1;
        Ok(())
    }

    pub fn add_entry(&mut self, name: String, inode: Arc<Mutex<RamInode>>) -> Result<(), FileError> {
        if self.file_type != FileType::Directory {
            return Err(FileError::NotDirectory);
        }

        if self.entries.contains_key(&name) {
            return Err(FileError::AlreadyExists);
        }

        self.entries.insert(name, inode);
        Ok(())
    }

    pub fn remove_entry(&mut self, name: &str) -> Result<(), FileError> {
        if self.file_type != FileType::Directory {
            return Err(FileError::NotDirectory);
        }

        self.entries.remove(name).ok_or(FileError::NotFound)?;
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Result<Arc<Mutex<RamInode>>, FileError> {
        if self.file_type != FileType::Directory {
            return Err(FileError::NotDirectory);
        }

        self.entries.get(name).cloned().ok_or(FileError::NotFound)
    }

    pub fn list_entries(&self) -> Result<Vec<String>, FileError> {
        if self.file_type != FileType::Directory {
            return Err(FileError::NotDirectory);
        }

        Ok(self.entries.keys().cloned().collect())
    }
}

impl Inode for RamInode {
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

/// RamFS文件句柄
pub struct RamFile {
    inode: Arc<Mutex<RamInode>>,
    offset: usize,
}

impl RamFile {
    pub fn new(inode: Arc<Mutex<RamInode>>) -> Self {
        RamFile { inode, offset: 0 }
    }
}

impl File for RamFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError> {
        let n = self.inode.lock().read_at(self.offset, buf)?;
        self.offset += n;
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        let n = self.inode.lock().write_at(self.offset, buf)?;
        self.offset += n;
        Ok(n)
    }

    fn seek(&mut self, pos: super::file::SeekFrom) -> Result<usize, FileError> {
        use super::file::SeekFrom;

        let size = self.inode.lock().size();

        let new_offset = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(delta) => {
                if delta >= 0 {
                    self.offset + delta as usize
                } else {
                    self.offset.saturating_sub((-delta) as usize)
                }
            }
            SeekFrom::End(delta) => {
                if delta >= 0 {
                    size + delta as usize
                } else {
                    size.saturating_sub((-delta) as usize)
                }
            }
        };

        self.offset = new_offset;
        Ok(self.offset)
    }

    fn size(&self) -> Result<usize, FileError> {
        Ok(self.inode.lock().size())
    }
}

/// RamFS文件系统
pub struct RamFS {
    root: Arc<Mutex<RamInode>>,
    next_ino: Mutex<usize>,
}

impl RamFS {
    pub fn new() -> Self {
        let root = Arc::new(Mutex::new(RamInode::new_directory(1)));
        RamFS {
            root,
            next_ino: Mutex::new(2),
        }
    }

    fn alloc_ino(&self) -> usize {
        let mut next = self.next_ino.lock();
        let ino = *next;
        *next += 1;
        ino
    }

    pub fn root(&self) -> Arc<Mutex<RamInode>> {
        self.root.clone()
    }

    pub fn create_file(&self, parent: Arc<Mutex<RamInode>>, name: String) -> Result<Arc<Mutex<RamInode>>, FileError> {
        let ino = self.alloc_ino();
        let inode = Arc::new(Mutex::new(RamInode::new_file(ino)));
        parent.lock().add_entry(name, inode.clone())?;
        Ok(inode)
    }

    pub fn create_directory(&self, parent: Arc<Mutex<RamInode>>, name: String) -> Result<Arc<Mutex<RamInode>>, FileError> {
        let ino = self.alloc_ino();
        let inode = Arc::new(Mutex::new(RamInode::new_directory(ino)));
        parent.lock().add_entry(name, inode.clone())?;
        Ok(inode)
    }

    pub fn remove(&self, parent: Arc<Mutex<RamInode>>, name: &str) -> Result<(), FileError> {
        parent.lock().remove_entry(name)
    }

    pub fn lookup(&self, parent: Arc<Mutex<RamInode>>, name: &str) -> Result<Arc<Mutex<RamInode>>, FileError> {
        parent.lock().lookup(name)
    }

    pub fn open_file(&self, inode: Arc<Mutex<RamInode>>) -> Result<RamFile, FileError> {
        let file_type = inode.lock().file_type();
        if file_type != FileType::RegularFile {
            return Err(FileError::IsDirectory);
        }
        Ok(RamFile::new(inode))
    }
}
