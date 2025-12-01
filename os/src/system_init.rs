//! System initialization - Create initial processes and files (with visualization)
//!
//! This is not demo code, but the real initialization process needed by the system
//! Also provides detailed visualization to let students see each step clearly

use crate::println;
use crate::fs::{RAMFS, File, Inode};
use crate::process::{create_process, SCHEDULER};
use alloc::string::String;

/// Delay function (for visualization demo)
/// Note: Delay is completely disabled for fast demonstration
fn delay() {
    // Delay completely disabled
}

/// Short delay (for quick demonstration)
/// Note: Delay is completely disabled for fast demonstration
fn short_delay() {
    // Delay completely disabled
}

/// Print separator line
fn print_separator() {
    println!("================================================================");
}

/// Print small separator line
fn print_small_separator() {
    println!("----------------------------------------");
}

/// Initialize system processes (with visualization)
///
/// Create real initial processes needed by the system (such as init process)
pub fn init_system_processes() {
    println!("\n[Chapter 6 Demo: Process Management]");
    print_separator();
    println!("Demo Content: Creating Real System Processes");
    print_separator();
    short_delay();

    // ========== Step 1: Create init process ==========
    println!("\n[Step 1] Create init process (PID 1, first system process)");
    print_small_separator();
    println!("  Configuration:");
    println!("    - Process name: init");
    println!("    - Entry address: 0x80000000");
    println!("    - Stack top: 0x80010000");
    println!("    - Parent process: None (this is the first process)");
    short_delay();

    let init_proc = create_process(
        "init",
        0x8000_0000,
        0x8001_0000,
        None,
    );
    SCHEDULER.lock().add_process(init_proc.clone());

    println!("\n  [OK] init process created successfully!");
    println!("    - PID: {}", init_proc.lock().pid().as_usize());
    println!("    - State: Ready");
    println!("    - Added to scheduling queue");
    short_delay();

    // Show current process state
    println!("\n  Current system process list:");
    crate::process::inspector::show_process_list();
    short_delay();

    // ========== Step 2: Create shell process ==========
    println!("\n[Step 2] Create shell process (user interaction interface)");
    print_small_separator();
    println!("  Configuration:");
    println!("    - Process name: shell");
    println!("    - Entry address: 0x80100000");
    println!("    - Stack top: 0x80110000");
    println!("    - Parent process: init (PID={})", init_proc.lock().pid().as_usize());
    short_delay();

    let shell_proc = create_process(
        "shell",
        0x8010_0000,
        0x8011_0000,
        Some(init_proc.lock().pid()),
    );
    SCHEDULER.lock().add_process(shell_proc.clone());

    println!("\n  [OK] shell process created successfully!");
    println!("    - PID: {}", shell_proc.lock().pid().as_usize());
    println!("    - Parent PID: {}", init_proc.lock().pid().as_usize());
    println!("    - State: Ready");
    short_delay();

    println!("\n  Current system process list:");
    crate::process::inspector::show_process_list();
    short_delay();

    // ========== Step 3: Create user processes ==========
    println!("\n[Step 3] Create user task processes");
    print_small_separator();

    for i in 1..=3 {
        let name = match i {
            1 => "user_task1",
            2 => "user_task2",
            3 => "user_task3",
            _ => "user_task",
        };

        println!("\n  Creating process: {}", name);
        println!("    - Entry address: {:#x}", 0x8020_0000 + i * 0x1000);
        println!("    - Stack top: {:#x}", 0x8021_0000 + i * 0x1000);
        println!("    - Parent process: init (PID={})", init_proc.lock().pid().as_usize());

        let proc = create_process(
            name,
            0x8020_0000 + i * 0x1000,
            0x8021_0000 + i * 0x1000,
            Some(init_proc.lock().pid()),
        );
        SCHEDULER.lock().add_process(proc.clone());

        println!("  [OK] {} created successfully (PID={})", name, proc.lock().pid().as_usize());
        short_delay();
    }

    println!("\n[Process creation complete] Final system state:");
    crate::process::inspector::show_system_dashboard();
    short_delay();
}

/// Initialize filesystem content (with visualization)
///
/// Create initial files and directories really needed by the system
pub fn init_filesystem_content() {
    println!("\n[Chapter 7 Demo: File System]");
    print_separator();
    println!("Demo Content: Creating Real Filesystem Structure");
    print_separator();
    short_delay();

    let root = RAMFS.root();

    println!("\n[Initial state] Filesystem is empty");
    crate::fs::inspector::show_filesystem_tree();
    short_delay();

    // ========== Step 1: Create /etc directory ==========
    println!("\n[Step 1] Create /etc directory (system config file directory)");
    print_small_separator();
    short_delay();

    match RAMFS.create_directory(root.clone(), String::from("etc")) {
        Ok(etc_dir) => {
            println!("  [OK] /etc directory created successfully");
            println!("    - Inode: {}", etc_dir.lock().ino());
            println!("    - Type: Directory");
            short_delay();

            println!("\n  Creating config files:");

            // Create passwd file
            if let Ok(passwd) = RAMFS.create_file(etc_dir.clone(), String::from("passwd")) {
                let mut file = RAMFS.open_file(passwd).unwrap();
                let content = b"root:x:0:0:root:/root:/bin/sh\n";
                file.write(content).ok();
                println!("    [OK] /etc/passwd");
                println!("      - Size: {} bytes", content.len());
                println!("      - Content: User account info");
                short_delay();
            }

            // Create hostname file
            if let Ok(hostname) = RAMFS.create_file(etc_dir.clone(), String::from("hostname")) {
                let mut file = RAMFS.open_file(hostname).unwrap();
                let content = b"error-os\n";
                file.write(content).ok();
                println!("    [OK] /etc/hostname");
                println!("      - Size: {} bytes", content.len());
                println!("      - Content: Hostname");
                short_delay();
            }

            println!("\n  Current filesystem state:");
            crate::fs::inspector::show_filesystem_tree();
            short_delay();
        }
        Err(e) => println!("  [ERR] Failed to create /etc: {:?}", e),
    }

    // Simplified creation of other directories (keep original logic)
    println!("\n[Step 2-4] Creating other directories and files...");
    short_delay();

    // /home
    if let Ok(home_dir) = RAMFS.create_directory(root.clone(), String::from("home")) {
        if let Ok(user_dir) = RAMFS.create_directory(home_dir, String::from("user")) {
            if let Ok(readme) = RAMFS.create_file(user_dir.clone(), String::from("README.txt")) {
                let mut file = RAMFS.open_file(readme).unwrap();
                file.write(b"Welcome to Error OS!\n").ok();
            }
        }
    }

    // /tmp
    if let Ok(tmp_dir) = RAMFS.create_directory(root.clone(), String::from("tmp")) {
        if let Ok(temp_file) = RAMFS.create_file(tmp_dir, String::from("test.log")) {
            let mut file = RAMFS.open_file(temp_file).unwrap();
            file.write(b"[INFO] System initialized\n").ok();
        }
    }

    // Root directory files
    if let Ok(version) = RAMFS.create_file(root.clone(), String::from("version")) {
        let mut file = RAMFS.open_file(version).unwrap();
        file.write(b"Error OS v0.1.0\n").ok();
    }

    if let Ok(motd) = RAMFS.create_file(root.clone(), String::from("motd")) {
        let mut file = RAMFS.open_file(motd).unwrap();
        file.write(b"Message of the Day: Welcome!\n").ok();
    }

    println!("\n[Filesystem creation complete] Final state:");
    crate::fs::inspector::show_filesystem_dashboard();
    short_delay();
}

/// Complete system initialization
pub fn initialize_system() {
    println!("\n=== System Initialization ===\n");

    init_system_processes();
    init_filesystem_content();

    println!("\n=== System Initialization Complete ===\n");
}
