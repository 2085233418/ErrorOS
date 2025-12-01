//! 文件系统检查器 - 真实文件系统状态查询和可视化
//!
//! 提供查询文件系统实际状态的接口：
//! - 列出所有文件和目录
//! - 查看FD表使用情况
//! - 显示文件系统树结构

use crate::println;
use super::{RAMFS, FD_TABLE, Inode};  // 添加Inode trait
use super::file::FileType;
use alloc::vec::Vec;
use alloc::string::String;

/// 文件/目录条目快照
#[derive(Clone)]
pub struct EntrySnapshot {
    pub name: String,
    pub ino: usize,
    pub file_type: FileType,
    pub size: usize,
}

/// FD表条目快照
#[derive(Clone)]
pub struct FdSnapshot {
    pub fd: usize,
    pub name: String,  // "Stdin", "Stdout", "Stderr", 或文件名
}

/// FD表统计信息
pub struct FdStats {
    pub total_fds: usize,
    pub stdin_fd: usize,
    pub stdout_fd: usize,
    pub stderr_fd: usize,
    pub file_fds: usize,
}

/// 获取根目录下所有文件和目录
pub fn get_root_entries() -> Vec<EntrySnapshot> {
    let mut entries = Vec::new();
    let root = RAMFS.root();

    // 获取根目录的所有条目
    if let Ok(entry_names) = root.lock().list_entries() {
        for name in entry_names {
            if let Ok(inode) = root.lock().lookup(&name) {
                let inode_guard = inode.lock();
                entries.push(EntrySnapshot {
                    name: name.clone(),
                    ino: inode_guard.ino(),
                    file_type: inode_guard.file_type(),
                    size: inode_guard.size(),
                });
            }
        }
    }

    entries
}

/// 获取已分配的FD列表
pub fn get_allocated_fds() -> Vec<FdSnapshot> {
    let mut fds = Vec::new();
    let fd_table = FD_TABLE.lock();

    // FD 0-2 是标准流
    if fd_table.get(0).is_some() {
        fds.push(FdSnapshot {
            fd: 0,
            name: "Stdin".into(),
        });
    }

    if fd_table.get(1).is_some() {
        fds.push(FdSnapshot {
            fd: 1,
            name: "Stdout".into(),
        });
    }

    if fd_table.get(2).is_some() {
        fds.push(FdSnapshot {
            fd: 2,
            name: "Stderr".into(),
        });
    }

    // FD >= 3 是用户文件
    for fd in 3..32 {  // 检查前32个FD
        if fd_table.get(fd).is_some() {
            fds.push(FdSnapshot {
                fd,
                name: alloc::format!("File-{}", fd),
            });
        }
    }

    fds
}

/// 获取FD表统计信息
pub fn get_fd_stats() -> FdStats {
    let fds = get_allocated_fds();

    let mut stdin = 0;
    let mut stdout = 0;
    let mut stderr = 0;
    let mut files = 0;

    for fd_snap in &fds {
        match fd_snap.fd {
            0 => stdin = 1,
            1 => stdout = 1,
            2 => stderr = 1,
            _ => files += 1,
        }
    }

    FdStats {
        total_fds: fds.len(),
        stdin_fd: stdin,
        stdout_fd: stdout,
        stderr_fd: stderr,
        file_fds: files,
    }
}

/// 可视化：显示根目录文件列表
pub fn show_file_list() {
    println!("\n================================================================");
    println!("===                Root Directory File List                  ===");
    println!("================================================================");

    let entries = get_root_entries();

    if entries.is_empty() {
        println!("===  (Root directory is empty)                               ===");
    } else {
        println!("===  Inode |  Name              |  Type    |  Size(B)      ===");
        println!("================================================================");

        for entry in entries {
            let type_str = match entry.file_type {
                FileType::RegularFile => "File ",
                FileType::Directory => "Dir  ",
                _ => "Other",
            };

            println!("===  {:4}  |  {:16} |  {}    |  {:6}        ===",
                     entry.ino, entry.name, type_str, entry.size);
        }
    }

    println!("================================================================");
}

/// 可视化：显示FD表使用情况
pub fn show_fd_table() {
    println!("\n================================================================");
    println!("===              File Descriptor Table                       ===");
    println!("================================================================");

    let fds = get_allocated_fds();

    if fds.is_empty() {
        println!("===  (FD table is empty)                                     ===");
    } else {
        println!("===   FD   |  File/Device                                   ===");
        println!("================================================================");

        for fd_snap in fds {
            println!("===   {:2}   |  {:20}                             ===",
                     fd_snap.fd, fd_snap.name);
        }
    }

    println!("================================================================");
}

/// 可视化：显示FD统计信息
pub fn show_fd_stats() {
    println!("\n================================================================");
    println!("===              FD Table Statistics                         ===");
    println!("================================================================");

    let stats = get_fd_stats();

    println!("===  Total FDs:     {:2}                                      ===", stats.total_fds);
    println!("===  Stdin (0):     {}                                       ===",
             if stats.stdin_fd > 0 { "Allocated" } else { "Not alloc" });
    println!("===  Stdout (1):    {}                                       ===",
             if stats.stdout_fd > 0 { "Allocated" } else { "Not alloc" });
    println!("===  Stderr (2):    {}                                       ===",
             if stats.stderr_fd > 0 { "Allocated" } else { "Not alloc" });
    println!("===  File FDs:      {:2}                                      ===", stats.file_fds);
    println!("================================================================");
}

/// 可视化：显示文件系统树
pub fn show_filesystem_tree() {
    println!("\n================================================================");
    println!("===              Filesystem Tree Structure                   ===");
    println!("================================================================");
    println!("===                                                          ===");
    println!("===  / (root, ino=1)                                         ===");

    let entries = get_root_entries();

    if entries.is_empty() {
        println!("===  (Empty directory)                                       ===");
    } else {
        for (i, entry) in entries.iter().enumerate() {
            let is_last = i == entries.len() - 1;
            let prefix = if is_last { "+--" } else { "|--" };

            let type_marker = match entry.file_type {
                FileType::Directory => "/",
                _ => "",
            };

            println!("===  {}  {} (ino={}, {}B){}                           ===",
                     prefix,
                     entry.name,
                     entry.ino,
                     entry.size,
                     type_marker);

            // 如果是目录，尝试列出子项
            if entry.file_type == FileType::Directory {
                if let Ok(inode) = RAMFS.root().lock().lookup(&entry.name) {
                    if let Ok(sub_entries) = inode.lock().list_entries() {
                        for (j, sub_name) in sub_entries.iter().enumerate() {
                            let is_sub_last = j == sub_entries.len() - 1;
                            let sub_prefix = if is_last { "    " } else { "|   " };
                            let sub_connector = if is_sub_last { "+--" } else { "|--" };

                            println!("===  {}{}  {}                                      ===",
                                     sub_prefix, sub_connector, sub_name);
                        }
                    }
                }
            }
        }
    }

    println!("===                                                          ===");
    println!("================================================================");
}

/// 可视化：完整的文件系统仪表盘
pub fn show_filesystem_dashboard() {
    println!("\n");
    println!("================================================================");
    println!("===                                                          ===");
    println!("===         Filesystem Real-time Monitoring Dashboard        ===");
    println!("===                                                          ===");
    println!("================================================================");

    show_fd_stats();
    show_fd_table();
    show_file_list();
    show_filesystem_tree();

    println!("");
}
