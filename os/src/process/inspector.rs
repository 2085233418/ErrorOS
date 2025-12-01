//! 进程检查器 - 真实系统状态查询和可视化
//!
//! 提供查询运行中系统状态的接口：
//! - 列出所有进程及其状态
//! - 查看进程详细信息
//! - 统计系统资源使用情况

use crate::println;
use super::scheduler::SCHEDULER;
use super::pcb::ProcessState;
use alloc::vec::Vec;
use alloc::string::String;

/// 进程快照 - 某一时刻的进程状态
#[derive(Clone)]
pub struct ProcessSnapshot {
    pub pid: usize,
    pub name: String,
    pub state: ProcessState,
    pub parent_pid: Option<usize>,
}

/// 系统统计信息
pub struct SystemStats {
    pub total_processes: usize,
    pub running_processes: usize,
    pub ready_processes: usize,
    pub blocked_processes: usize,
    pub zombie_processes: usize,
}

/// 获取所有进程的快照
pub fn get_all_processes() -> Vec<ProcessSnapshot> {
    let scheduler = SCHEDULER.lock();
    let mut snapshots = Vec::new();

    // 遍历调度器中的所有进程
    for (pid, process_handle) in scheduler.processes() {
        let pcb = process_handle.lock();
        snapshots.push(ProcessSnapshot {
            pid: (*pid).as_usize(),  // 转换ProcessId到usize
            name: pcb.name().into(),
            state: pcb.state(),
            parent_pid: pcb.parent_pid().map(|p| p.as_usize()),  // 转换Option<ProcessId>
        });
    }

    snapshots
}

/// 获取系统统计信息
pub fn get_system_stats() -> SystemStats {
    let processes = get_all_processes();

    let mut running = 0;
    let mut ready = 0;
    let mut blocked = 0;
    let mut zombie = 0;

    for proc in &processes {
        match proc.state {
            ProcessState::Running => running += 1,
            ProcessState::Ready => ready += 1,
            ProcessState::Blocked => blocked += 1,
            ProcessState::Zombie => zombie += 1,
        }
    }

    SystemStats {
        total_processes: processes.len(),
        running_processes: running,
        ready_processes: ready,
        blocked_processes: blocked,
        zombie_processes: zombie,
    }
}

/// 获取当前正在运行的进程信息
pub fn get_current_process() -> Option<ProcessSnapshot> {
    let scheduler = SCHEDULER.lock();

    if let Some(current_pid) = scheduler.current_pid() {
        if let Some(process_handle) = scheduler.get_process(current_pid) {
            let pcb = process_handle.lock();
            return Some(ProcessSnapshot {
                pid: current_pid.as_usize(),  // 转换ProcessId
                name: pcb.name().into(),
                state: pcb.state(),
                parent_pid: pcb.parent_pid().map(|p| p.as_usize()),  // 转换Option<ProcessId>
            });
        }
    }

    None
}

/// 可视化：显示所有进程列表
pub fn show_process_list() {
    println!("\n================================================================");
    println!("===                  System Process List                     ===");
    println!("================================================================");

    let processes = get_all_processes();

    if processes.is_empty() {
        println!("===  (No processes in system)                                ===");
    } else {
        println!("===  PID  |  Name              |  State      |  Parent PID  ===");
        println!("================================================================");

        for proc in processes {
            let state_str = match proc.state {
                ProcessState::Running => "Running   ",
                ProcessState::Ready => "Ready     ",
                ProcessState::Blocked => "Blocked   ",
                ProcessState::Zombie => "Zombie    ",
            };

            let parent_str = match proc.parent_pid {
                Some(ppid) => alloc::format!("{:6}", ppid),
                None => "  -   ".into(),
            };

            println!("===  {:3}  |  {:16} |  {}  |  {:8}       ===",
                     proc.pid, proc.name, state_str, parent_str);
        }
    }

    println!("================================================================");
}

/// 可视化：显示系统统计信息
pub fn show_system_stats() {
    println!("\n================================================================");
    println!("===                  System Statistics                       ===");
    println!("================================================================");

    let stats = get_system_stats();

    println!("===  Total Processes:   {:3}                                 ===", stats.total_processes);
    println!("===  Running:           {:3}                                 ===", stats.running_processes);
    println!("===  Ready:             {:3}                                 ===", stats.ready_processes);
    println!("===  Blocked:           {:3}                                 ===", stats.blocked_processes);
    println!("===  Zombie:            {:3}                                 ===", stats.zombie_processes);
    println!("================================================================");
}

/// 可视化：显示当前进程信息
pub fn show_current_process() {
    println!("\n================================================================");
    println!("===                  Current Running Process                 ===");
    println!("================================================================");

    if let Some(proc) = get_current_process() {
        println!("===  PID:         {:3}                                       ===", proc.pid);
        println!("===  Name:        {:16}                             ===", proc.name);
        println!("===  State:       Running                                    ===");
        println!("===  Parent PID:  {:3}                                       ===",
                 proc.parent_pid.unwrap_or(0));
    } else {
        println!("===  (No process running - IDLE state)                       ===");
    }

    println!("================================================================");
}

/// 可视化：完整的系统状态仪表盘
pub fn show_system_dashboard() {
    println!("\n");
    println!("================================================================");
    println!("===                                                          ===");
    println!("===          OS Real-time Monitoring Dashboard               ===");
    println!("===                                                          ===");
    println!("================================================================");

    show_system_stats();
    show_current_process();
    show_process_list();

    println!("");
}
