/*
 * ============================================
 * 进程管理模块（Process Management）
 * ============================================
 * 功能：提供完整的进程管理功能
 *
 * 核心组件：
 * - PID分配器：分配唯一进程ID
 * - 进程控制块（PCB）：管理进程信息
 * - 上下文切换：保存/恢复进程状态
 * - 调度器：管理进程调度
 *
 * 使用方式：
 * 1. 调用 init() 初始化进程系统
 * 2. 使用 create_process() 创建进程
 * 3. 调用 scheduler::start_scheduling() 开始调度
 * 4. 在时钟中断中调用 scheduler::tick()
 * ============================================
 */

// 引入汇编代码
core::arch::global_asm!(include_str!("switch.S"));

// ============================================
// 子模块
// ============================================

pub mod pid;
pub mod context;
pub mod pcb;
pub mod scheduler;
pub mod inspector;      // 真实系统状态查询模块

// ============================================
// 重新导出核心类型
// ============================================

pub use pid::ProcessId;
pub use context::ProcessContext;
pub use pcb::{
    ProcessControlBlock,
    ProcessState,
    ProcessHandle,
    create_process_handle,
};
pub use scheduler::SCHEDULER;

use crate::serial_println;

// ============================================
// 初始化
// ============================================

/// 初始化进程管理系统
///
/// # 说明
/// - 初始化PID分配器
/// - 初始化调度器
/// - 准备创建init进程
pub fn init() {
    serial_println!("[PROCESS] Initializing process management system");

    // 初始化调度器
    scheduler::init();

    serial_println!("[PROCESS] Process management system initialized");
}

// ============================================
// 进程创建
// ============================================

/// 创建新进程
///
/// # 参数
/// - `name`: 进程名称
/// - `entry_point`: 程序入口地址
/// - `user_stack_top`: 用户栈顶地址
/// - `parent_pid`: 父进程PID（None表示init进程）
///
/// # 返回
/// 新创建的进程句柄
///
/// # 说明
/// 1. 分配PID
/// 2. 创建PCB
/// 3. 初始化上下文
/// 4. 设置用户栈和页表
/// 5. 加入调度器
pub fn create_process(
    name: &'static str,
    entry_point: usize,
    user_stack_top: usize,
    parent_pid: Option<ProcessId>,
) -> ProcessHandle {
    // 注释掉调试输出，避免刷屏
    // serial_println!(
    //     "[PROCESS] Creating process: {} (entry={:#x}, stack={:#x})",
    //     name,
    //     entry_point,
    //     user_stack_top
    // );

    // 创建PCB
    let process = create_process_handle(name, parent_pid);

    // 初始化上下文
    {
        let mut pcb = process.lock();

        // 设置用户栈
        pcb.set_user_stack(user_stack_top - 0x10000, user_stack_top);

        // 创建用户态上下文
        // 注意：当前使用恒等映射（identity mapping），即虚拟地址=物理地址
        // 在第7章实现完整的地址空间后，这里会获取进程的实际页表基址
        let satp_value = 0;  // 恒等映射模式下，satp=0表示不使用分页
        let context = ProcessContext::new_user_context(
            entry_point,
            user_stack_top,
            satp_value,
        );
        *pcb.context_mut() = context;
    }

    // serial_println!("[PROCESS] Process created: PID={}", process.lock().pid());

    process
}

// ============================================
// 进程控制
// ============================================

/// 退出当前进程
///
/// # 参数
/// - `exit_code`: 退出码
///
/// # 说明
/// 1. 设置进程状态为 Zombie
/// 2. 保存退出码
/// 3. 通知父进程
/// 4. 触发调度
pub fn exit_current_process(exit_code: i32) {
    let current = scheduler::current_process();

    if let Some(process) = current {
        let pid = process.lock().pid();
        serial_println!("[PROCESS] Process PID={} exiting with code {}", pid, exit_code);

        // 设置退出码和状态
        process.lock().set_exit_code(exit_code);

        // TODO: 通知父进程
        // TODO: 回收资源（页表、内存等）

        // 触发调度
        scheduler::SCHEDULER.lock().schedule();
    }
}

/// 阻塞当前进程
pub fn block_current_process() {
    scheduler::SCHEDULER.lock().block_current();
}

/// 唤醒进程
pub fn wake_up_process(pid: ProcessId) {
    scheduler::SCHEDULER.lock().wake_up(pid);
}

// ============================================
// 查询接口
// ============================================

/// 获取当前进程PID
pub fn current_pid() -> Option<ProcessId> {
    scheduler::current_pid()
}

/// 获取当前进程句柄
pub fn current_process() -> Option<ProcessHandle> {
    scheduler::current_process()
}

// ============================================
// 调试
// ============================================

/// 打印进程信息
pub fn print_process_info() {
    scheduler::print_status();
}

// ============================================
// 测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_process_creation() {
        init();

        let process = create_process("test", 0x1000, 0x2000, None);
        let pid = process.lock().pid();

        assert!(pid.as_usize() > 0);
    }

    #[test_case]
    fn test_process_state_transition() {
        init();

        let process = create_process("test", 0x1000, 0x2000, None);

        {
            let mut pcb = process.lock();
            assert_eq!(pcb.state(), ProcessState::Ready);

            pcb.set_state(ProcessState::Running);
            assert_eq!(pcb.state(), ProcessState::Running);
        }
    }
}
