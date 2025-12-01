/*
 * ============================================
 * 进程控制块（Process Control Block, PCB）
 * ============================================
 * 功能：管理单个进程的所有信息
 *
 * PCB包含：
 * - 基础信息：PID、父进程、状态
 * - 执行上下文：寄存器状态
 * - 内存信息：地址空间、堆、栈
 * - 调度信息：时间片、优先级
 * - 退出信息：退出码、子进程列表
 * ============================================
 */

extern crate alloc;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

use super::pid::ProcessId;
use super::context::ProcessContext;
use crate::memory::AddressSpace;

// ============================================
// 进程状态
// ============================================

/// 进程状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// 就绪态：等待被调度执行
    Ready,

    /// 运行态：正在CPU上执行
    Running,

    /// 阻塞态：等待某个事件（如I/O、等待子进程）
    Blocked,

    /// 僵尸态：已退出但未被父进程回收
    /// 保留PCB以便父进程获取退出码
    Zombie,
}

impl core::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ProcessState::Ready => write!(f, "Ready"),
            ProcessState::Running => write!(f, "Running"),
            ProcessState::Blocked => write!(f, "Blocked"),
            ProcessState::Zombie => write!(f, "Zombie"),
        }
    }
}

// ============================================
// 进程控制块
// ============================================

/// 进程控制块（PCB）
///
/// # 说明
/// - 每个进程有唯一的 PCB
/// - PCB 通过 Arc<Mutex<>> 包装以支持多核共享
/// - 调度器维护一个 PID -> PCB 的映射表
pub struct ProcessControlBlock {
    // ============================================
    // 基础信息
    // ============================================

    /// 进程ID（全局唯一）
    pid: ProcessId,

    /// 父进程ID（None表示init进程）
    parent_pid: Option<ProcessId>,

    /// 进程状态
    state: ProcessState,

    /// 进程名称（用于调试）
    name: &'static str,

    // ============================================
    // 执行上下文
    // ============================================

    /// 进程上下文（寄存器状态）
    /// 在上下文切换时保存/恢复
    context: ProcessContext,

    // ============================================
    // 内存信息
    // ============================================

    /// 地址空间（页表）
    /// None 表示使用内核地址空间（仅限内核线程）
    address_space: Option<AddressSpace>,

    /// 堆底地址（动态内存起始）
    heap_bottom: usize,

    /// 堆顶地址（当前堆的结束位置）
    heap_top: usize,

    /// 用户栈底地址
    user_stack_bottom: usize,

    /// 用户栈顶地址
    user_stack_top: usize,

    // ============================================
    // 调度信息
    // ============================================

    /// 剩余时间片（时钟中断计数）
    time_slice: usize,

    /// 优先级（数值越大优先级越高，暂时未使用）
    priority: usize,

    // ============================================
    // 进程关系
    // ============================================

    /// 子进程列表
    children: Vec<ProcessId>,

    /// 退出码（Some表示已退出）
    exit_code: Option<i32>,
}

impl ProcessControlBlock {
    /// 创建一个新的进程控制块
    ///
    /// # 参数
    /// - `name`: 进程名称
    /// - `parent_pid`: 父进程ID
    ///
    /// # 返回
    /// 新创建的 PCB，状态为 Ready
    pub fn new(name: &'static str, parent_pid: Option<ProcessId>) -> Self {
        ProcessControlBlock {
            pid: ProcessId::new(),
            parent_pid,
            state: ProcessState::Ready,
            name,
            context: ProcessContext::new(),
            address_space: None,
            heap_bottom: 0,
            heap_top: 0,
            user_stack_bottom: 0,
            user_stack_top: 0,
            time_slice: 5,  // 默认时间片：5个时钟周期
            priority: 1,     // 默认优先级
            children: Vec::new(),
            exit_code: None,
        }
    }

    // ============================================
    // Getter 方法
    // ============================================

    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    pub fn parent_pid(&self) -> Option<ProcessId> {
        self.parent_pid
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn context(&self) -> &ProcessContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut ProcessContext {
        &mut self.context
    }

    pub fn address_space(&self) -> Option<&AddressSpace> {
        self.address_space.as_ref()
    }

    pub fn children(&self) -> &Vec<ProcessId> {
        &self.children
    }

    // ============================================
    // Setter 方法
    // ============================================

    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    pub fn set_address_space(&mut self, space: AddressSpace) {
        self.address_space = Some(space);
    }

    pub fn set_user_stack(&mut self, bottom: usize, top: usize) {
        self.user_stack_bottom = bottom;
        self.user_stack_top = top;
    }

    pub fn set_heap(&mut self, bottom: usize) {
        self.heap_bottom = bottom;
        self.heap_top = bottom;
    }

    pub fn set_exit_code(&mut self, code: i32) {
        self.exit_code = Some(code);
        self.state = ProcessState::Zombie;
    }

    // ============================================
    // 进程关系管理
    // ============================================

    /// 添加子进程
    pub fn add_child(&mut self, child_pid: ProcessId) {
        self.children.push(child_pid);
    }

    /// 移除子进程
    pub fn remove_child(&mut self, child_pid: ProcessId) {
        self.children.retain(|&pid| pid != child_pid);
    }

    // ============================================
    // 调度相关
    // ============================================

    /// 重置时间片
    pub fn reset_time_slice(&mut self) {
        self.time_slice = 5;
    }

    /// 减少时间片
    ///
    /// # 返回
    /// - `true`: 时间片用完，需要调度
    /// - `false`: 还有剩余时间片
    pub fn tick(&mut self) -> bool {
        if self.time_slice > 0 {
            self.time_slice -= 1;
        }
        self.time_slice == 0
    }

    // ============================================
    // 状态检查
    // ============================================

    pub fn is_ready(&self) -> bool {
        self.state == ProcessState::Ready
    }

    pub fn is_running(&self) -> bool {
        self.state == ProcessState::Running
    }

    pub fn is_blocked(&self) -> bool {
        self.state == ProcessState::Blocked
    }

    pub fn is_zombie(&self) -> bool {
        self.state == ProcessState::Zombie
    }

    /// 检查是否可以被回收
    ///
    /// # 条件
    /// - 状态为 Zombie
    /// - 已被父进程等待
    pub fn can_reclaim(&self) -> bool {
        self.is_zombie()
    }
}

// ============================================
// Debug 实现
// ============================================

impl core::fmt::Debug for ProcessControlBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("ProcessControlBlock")
            .field("pid", &self.pid)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("parent_pid", &self.parent_pid)
            .field("time_slice", &self.time_slice)
            .field("children_count", &self.children.len())
            .field("exit_code", &self.exit_code)
            .finish()
    }
}

// ============================================
// 类型别名
// ============================================

/// 进程句柄（支持多核共享）
pub type ProcessHandle = Arc<Mutex<ProcessControlBlock>>;

// ============================================
// 辅助函数
// ============================================

/// 创建进程句柄
pub fn create_process_handle(name: &'static str, parent_pid: Option<ProcessId>) -> ProcessHandle {
    Arc::new(Mutex::new(ProcessControlBlock::new(name, parent_pid)))
}

// ============================================
// 测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_pcb_creation() {
        let pcb = ProcessControlBlock::new("test", None);
        assert_eq!(pcb.name(), "test");
        assert_eq!(pcb.state(), ProcessState::Ready);
        assert!(pcb.parent_pid().is_none());
    }

    #[test_case]
    fn test_pcb_state_transition() {
        let mut pcb = ProcessControlBlock::new("test", None);

        pcb.set_state(ProcessState::Running);
        assert!(pcb.is_running());

        pcb.set_state(ProcessState::Blocked);
        assert!(pcb.is_blocked());

        pcb.set_exit_code(0);
        assert!(pcb.is_zombie());
        assert_eq!(pcb.exit_code(), Some(0));
    }

    #[test_case]
    fn test_pcb_time_slice() {
        let mut pcb = ProcessControlBlock::new("test", None);
        pcb.reset_time_slice();

        // 消耗时间片
        for _ in 0..4 {
            assert!(!pcb.tick());
        }

        // 最后一个tick应该返回true
        assert!(pcb.tick());
    }

    #[test_case]
    fn test_pcb_children_management() {
        let mut parent = ProcessControlBlock::new("parent", None);
        let child_pid = ProcessId::new();

        parent.add_child(child_pid);
        assert_eq!(parent.children().len(), 1);

        parent.remove_child(child_pid);
        assert_eq!(parent.children().len(), 0);
    }
}
