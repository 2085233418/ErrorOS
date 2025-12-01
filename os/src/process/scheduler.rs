/*
 * ============================================
 * 进程调度器（Process Scheduler）
 * ============================================
 * 功能：管理进程调度和切换
 *
 * 调度算法：Round-Robin（时间片轮转）
 * - 每个进程分配固定时间片
 * - 时间片用完后切换到下一个就绪进程
 * - 公平调度，避免饥饿
 *
 * 数据结构：
 * - 进程表：所有进程的PCB（PID -> PCB映射）
 * - 就绪队列：等待执行的进程PID列表
 * - 当前进程：正在执行的进程PID
 * ============================================
 */

extern crate alloc;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::lazy_static;

use super::pid::ProcessId;
use super::pcb::{ProcessState, ProcessHandle};
use super::context::{ProcessContext, switch_context};

use crate::serial_println;

// ============================================
// 调试输出开关
// ============================================
// 设置为false可以禁用scheduler的调试输出，避免刷屏
const ENABLE_SCHEDULER_DEBUG: bool = false;

macro_rules! scheduler_debug {
    ($($arg:tt)*) => {
        // 调试输出已禁用，避免刷屏
        // 如果需要启用，取消注释下面这行：
        // serial_println!($($arg)*);
    };
}

// ============================================
// 全局调度器实例
// ============================================

lazy_static! {
    /// 全局调度器
    ///
    /// 使用 lazy_static 确保在运行时初始化
    /// 使用 Mutex 保证线程安全
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

// ============================================
// 调度器结构
// ============================================

pub struct Scheduler {
    /// 进程表：PID -> PCB 映射
    ///
    /// 使用 BTreeMap 保证 PID 有序
    /// 使用 Arc<Mutex<>> 允许多处共享 PCB
    processes: BTreeMap<ProcessId, ProcessHandle>,

    /// 就绪队列（Round-Robin队列）
    ///
    /// 存储等待执行的进程PID
    /// 队首是下一个要执行的进程
    ready_queue: VecDeque<ProcessId>,

    /// 当前运行的进程PID
    ///
    /// None 表示没有进程在运行（idle状态）
    current: Option<ProcessId>,
}

impl Scheduler {
    /// 创建新的调度器
    pub fn new() -> Self {
        Scheduler {
            processes: BTreeMap::new(),
            ready_queue: VecDeque::new(),
            current: None,
        }
    }

    // ============================================
    // 进程管理
    // ============================================

    /// 添加新进程到调度器
    ///
    /// # 参数
    /// - `process`: 进程句柄
    ///
    /// # 说明
    /// - 将进程加入进程表
    /// - 如果进程状态为 Ready，加入就绪队列
    pub fn add_process(&mut self, process: ProcessHandle) {
        let pid = process.lock().pid();
        let state = process.lock().state();

        scheduler_debug!("[SCHEDULER] Add process: PID={}, State={:?}", pid, state);

        // 加入进程表
        self.processes.insert(pid, process.clone());

        // 如果进程就绪，加入就绪队列
        if state == ProcessState::Ready {
            self.ready_queue.push_back(pid);
            scheduler_debug!("[SCHEDULER] Process PID={} added to ready queue", pid);
        }
    }

    /// 移除进程
    ///
    /// # 参数
    /// - `pid`: 要移除的进程PID
    ///
    /// # 说明
    /// - 从进程表移除
    /// - 从就绪队列移除
    /// - 如果是当前进程，清空 current
    pub fn remove_process(&mut self, pid: ProcessId) {
        scheduler_debug!("[SCHEDULER] Remove process: PID={}", pid);

        // 从就绪队列移除
        self.ready_queue.retain(|&p| p != pid);

        // 从进程表移除
        self.processes.remove(&pid);

        // 如果是当前进程，清空
        if self.current == Some(pid) {
            self.current = None;
        }
    }

    /// 获取进程句柄
    pub fn get_process(&self, pid: ProcessId) -> Option<ProcessHandle> {
        self.processes.get(&pid).cloned()
    }

    /// 获取当前进程PID
    pub fn current_pid(&self) -> Option<ProcessId> {
        self.current
    }

    /// 获取当前进程句柄
    pub fn current_process(&self) -> Option<ProcessHandle> {
        self.current.and_then(|pid| self.get_process(pid))
    }

    /// 获取所有进程的迭代器（用于状态检查和可视化）
    pub fn processes(&self) -> impl Iterator<Item = (&ProcessId, &ProcessHandle)> {
        self.processes.iter()
    }

    // ============================================
    // 调度核心
    // ============================================

    /// 选择下一个要执行的进程
    ///
    /// # 返回
    /// - Some(pid): 下一个进程的PID
    /// - None: 没有就绪进程
    ///
    /// # Round-Robin 算法
    /// 1. 从就绪队列头取出一个进程
    /// 2. 如果队列为空，返回 None
    fn pick_next(&mut self) -> Option<ProcessId> {
        self.ready_queue.pop_front()
    }

    /// 将进程放回就绪队列
    ///
    /// # 参数
    /// - `pid`: 进程PID
    ///
    /// # 说明
    /// 用于时间片用完的进程
    fn enqueue(&mut self, pid: ProcessId) {
        // 检查进程状态
        if let Some(process) = self.get_process(pid) {
            let state = process.lock().state();
            if state == ProcessState::Ready {
                self.ready_queue.push_back(pid);
                scheduler_debug!("[SCHEDULER] Process PID={} enqueued", pid);
            }
        }
    }

    /// 调度新进程
    ///
    /// # 说明
    /// 1. 保存当前进程上下文
    /// 2. 选择下一个进程
    /// 3. 恢复下一个进程上下文
    /// 4. 执行上下文切换
    pub fn schedule(&mut self) {
        // 选择下一个进程
        let next_pid = match self.pick_next() {
            Some(pid) => pid,
            None => {
                // 没有就绪进程，保持当前进程或进入 idle
                scheduler_debug!("[SCHEDULER] No ready process, staying idle");
                return;
            }
        };

        let next_process = match self.get_process(next_pid) {
            Some(p) => p,
            None => {
                scheduler_debug!("[SCHEDULER] ERROR: Process PID={} not found", next_pid);
                return;
            }
        };

        // 获取当前进程
        let current_pid = self.current;

        // 如果下一个进程就是当前进程，无需切换
        if Some(next_pid) == current_pid {
            let mut next = next_process.lock();
            next.set_state(ProcessState::Running);
            next.reset_time_slice();
            drop(next);
            return;
        }

        scheduler_debug!(
            "[SCHEDULER] Context switch: {:?} -> {}",
            current_pid,
            next_pid
        );

        // 执行上下文切换
        match current_pid {
            Some(current_pid) => {
                // 有当前进程，需要保存状态
                let current_process = self.get_process(current_pid).unwrap();
                self.switch_to(current_process, next_process, next_pid);
            }
            None => {
                // 没有当前进程（初次调度），直接启动新进程
                self.start_process(next_process, next_pid);
            }
        }
    }

    /// 从当前进程切换到新进程
    fn switch_to(
        &mut self,
        current_process: ProcessHandle,
        next_process: ProcessHandle,
        next_pid: ProcessId,
    ) {
        let mut current = current_process.lock();
        let mut next = next_process.lock();

        // 更新进程状态
        if current.state() == ProcessState::Running {
            current.set_state(ProcessState::Ready);
            // 将当前进程放回就绪队列（时间片轮转）
            let current_pid = current.pid();
            drop(current);
            drop(next);
            self.enqueue(current_pid);

            // 重新获取锁
            current = current_process.lock();
            next = next_process.lock();
        }

        next.set_state(ProcessState::Running);
        next.reset_time_slice();

        // 更新当前进程
        self.current = Some(next_pid);

        // 获取上下文指针
        let current_ctx = current.context_mut() as *mut ProcessContext;
        let next_ctx = next.context() as *const ProcessContext;

        // 释放锁（避免死锁）
        drop(current);
        drop(next);

        // 执行上下文切换（汇编实现）
        unsafe {
            switch_context(current_ctx, next_ctx);
        }

        // 注意：这里不会返回，直到下次调度回到此进程
    }

    /// 启动新进程（首次调度）
    fn start_process(&mut self, next_process: ProcessHandle, next_pid: ProcessId) {
        let mut next = next_process.lock();

        next.set_state(ProcessState::Running);
        next.reset_time_slice();

        self.current = Some(next_pid);

        scheduler_debug!("[SCHEDULER] Starting first process: PID={}", next_pid);

        // 获取上下文
        let next_ctx = next.context() as *const ProcessContext;

        drop(next);

        // 进入新进程（不保存当前上下文）
        // TODO: 使用 enter_user_mode 或类似机制
        unsafe {
            // 临时实现：直接跳转
            // 完整实现需要使用 sret 进入用户态
            core::arch::asm!(
                "mv sp, {0}",
                "ret",
                in(reg) (*next_ctx).sp,
            );
        }
    }

    // ============================================
    // 时钟中断处理
    // ============================================

    /// 时钟中断回调
    ///
    /// # 说明
    /// 在时钟中断处理函数中调用
    /// 减少当前进程时间片，时间片用完时触发调度
    pub fn tick(&mut self) {
        if let Some(current_pid) = self.current {
            if let Some(process) = self.get_process(current_pid) {
                let mut pcb = process.lock();

                // 减少时间片
                let should_schedule = pcb.tick();

                if should_schedule {
                    scheduler_debug!("[SCHEDULER] Time slice expired for PID={}", current_pid);
                    drop(pcb);

                    // 触发调度
                    self.schedule();
                }
            }
        }
    }

    // ============================================
    // 进程状态转换
    // ============================================

    /// 阻塞当前进程
    ///
    /// # 说明
    /// 将当前进程状态设置为 Blocked，触发调度
    pub fn block_current(&mut self) {
        if let Some(current_pid) = self.current {
            if let Some(process) = self.get_process(current_pid) {
                let mut pcb = process.lock();
                pcb.set_state(ProcessState::Blocked);
                drop(pcb);

                scheduler_debug!("[SCHEDULER] Process PID={} blocked", current_pid);

                // 触发调度
                self.schedule();
            }
        }
    }

    /// 唤醒进程
    ///
    /// # 参数
    /// - `pid`: 要唤醒的进程PID
    ///
    /// # 说明
    /// 将进程状态从 Blocked 改为 Ready，加入就绪队列
    pub fn wake_up(&mut self, pid: ProcessId) {
        if let Some(process) = self.get_process(pid) {
            let mut pcb = process.lock();
            if pcb.state() == ProcessState::Blocked {
                pcb.set_state(ProcessState::Ready);
                drop(pcb);

                self.enqueue(pid);
                scheduler_debug!("[SCHEDULER] Process PID={} woken up", pid);
            }
        }
    }

    // ============================================
    // 调试
    // ============================================

    /// 打印调度器状态
    pub fn print_status(&self) {
        scheduler_debug!("\n========================================");
        scheduler_debug!("  调度器状态");
        scheduler_debug!("========================================");
        scheduler_debug!("当前进程: {:?}", self.current);
        scheduler_debug!("就绪队列: {:?}", self.ready_queue);
        scheduler_debug!("进程总数: {}", self.processes.len());

        for (pid, process) in &self.processes {
            let pcb = process.lock();
            scheduler_debug!(
                "  PID={}: {} [{}]",
                pid,
                pcb.name(),
                pcb.state()
            );
        }
        scheduler_debug!("========================================\n");
    }
}

// ============================================
// 全局接口函数
// ============================================

/// 初始化调度器
pub fn init() {
    scheduler_debug!("[SCHEDULER] Initializing scheduler");
}

/// 添加进程到全局调度器
pub fn add_process(process: ProcessHandle) {
    SCHEDULER.lock().add_process(process);
}

/// 启动调度
pub fn start_scheduling() {
    scheduler_debug!("[SCHEDULER] Starting scheduling");
    SCHEDULER.lock().schedule();
}

/// 时钟中断回调
pub fn tick() {
    SCHEDULER.lock().tick();
}

/// 获取当前进程PID
pub fn current_pid() -> Option<ProcessId> {
    SCHEDULER.lock().current_pid()
}

/// 获取当前进程句柄
pub fn current_process() -> Option<ProcessHandle> {
    SCHEDULER.lock().current_process()
}

/// 打印调度器状态
pub fn print_status() {
    SCHEDULER.lock().print_status();
}
