/*
 * ============================================
 * 系统调用具体实现
 * ============================================
 */

use crate::serial_println;
use crate::fs::{RAMFS, FD_TABLE};
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;

/// sys_write - 写入数据到文件描述符
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if buf.is_null() {
        return -1;
    }

    let slice = unsafe { core::slice::from_raw_parts(buf, len) };

    // 获取文件并写入
    match FD_TABLE.lock().get(fd) {
        Some(file) => match file.lock().write(slice) {
            Ok(n) => n as isize,
            Err(_) => -1,
        },
        None => {
            serial_println!("[SYSCALL] sys_write: invalid fd={}", fd);
            -1
        }
    }
}

/// sys_read - 从文件描述符读取数据
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    if buf.is_null() {
        return -1;
    }

    let buffer = unsafe { core::slice::from_raw_parts_mut(buf, len) };

    // 获取文件并读取
    match FD_TABLE.lock().get(fd) {
        Some(file) => match file.lock().read(buffer) {
            Ok(n) => n as isize,
            Err(_) => -1,
        },
        None => -1,
    }
}

/// sys_open - 打开文件
pub fn sys_open(path: *const u8, flags: usize) -> isize {
    if path.is_null() {
        return -1;
    }

    // 读取路径字符串
    let path_str = unsafe {
        let mut len = 0;
        while *path.add(len) != 0 {
            len += 1;
            if len > 256 {
                return -1;
            }
        }
        let slice = core::slice::from_raw_parts(path, len);
        match core::str::from_utf8(slice) {
            Ok(s) => String::from(s),
            Err(_) => return -1,
        }
    };

    // 在根目录查找或创建文件
    let root = RAMFS.root();
    let inode = {
        let root_guard = root.lock();
        match root_guard.lookup(&path_str) {
            Ok(inode) => inode,
            Err(_) => {
                drop(root_guard);
                // 文件不存在，创建新文件
                match RAMFS.create_file(root.clone(), path_str) {
                    Ok(inode) => inode,
                    Err(_) => return -1,
                }
            }
        }
    };

    // 打开文件
    match RAMFS.open_file(inode) {
        Ok(file) => {
            let file_arc: Arc<Mutex<dyn crate::fs::File>> = Arc::new(Mutex::new(file));
            match FD_TABLE.lock().alloc(file_arc) {
                Some(fd) => fd as isize,
                None => -1,
            }
        }
        Err(_) => -1,
    }
}

/// sys_close - 关闭文件描述符
pub fn sys_close(fd: usize) -> isize {
    if FD_TABLE.lock().dealloc(fd) {
        0
    } else {
        -1
    }
}

/// sys_mkdir - 创建目录
pub fn sys_mkdir(path: *const u8) -> isize {
    if path.is_null() {
        return -1;
    }

    let path_str = unsafe {
        let mut len = 0;
        while *path.add(len) != 0 {
            len += 1;
            if len > 256 {
                return -1;
            }
        }
        let slice = core::slice::from_raw_parts(path, len);
        match core::str::from_utf8(slice) {
            Ok(s) => String::from(s),
            Err(_) => return -1,
        }
    };

    let root = RAMFS.root();
    match RAMFS.create_directory(root, path_str) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// sys_exit - 退出进程
pub fn sys_exit(exit_code: i32) -> isize {
    serial_println!("[SYSCALL] sys_exit({})", exit_code);
    loop {}
}

/// sys_getpid - 获取当前进程ID
pub fn sys_getpid() -> isize {
    1
}

/// sys_fork - 创建子进程
pub fn sys_fork() -> isize {
    serial_println!("[SYSCALL] sys_fork: not implemented yet");
    -1
}

/// sys_exec - 执行程序
pub fn sys_exec(path: *const u8) -> isize {
    serial_println!("[SYSCALL] sys_exec: not implemented yet");
    -1
}

/// sys_waitpid - 等待子进程退出
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    serial_println!("[SYSCALL] sys_waitpid: not implemented yet");
    -1
}
