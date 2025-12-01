/*
 * ============================================
 * 进程 ID (PID) 分配器
 * ============================================
 * 功能：为新创建的进程分配唯一的进程ID
 *
 * 设计要点：
 * - 使用原子计数器确保PID唯一性
 * - PID从1开始（0保留给内核）
 * - 线程安全，支持多核环境
 * ============================================
 */

use core::sync::atomic::{AtomicUsize, Ordering};

/// 进程ID类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(usize);

impl ProcessId {
    /// 创建一个新的进程ID
    ///
    /// # 说明
    /// - 使用原子递增确保唯一性
    /// - 从1开始（init进程）
    /// - 理论最大值：usize::MAX
    pub fn new() -> Self {
        static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

        let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);

        ProcessId(pid)
    }

    /// 从数字创建PID（仅用于特殊情况，如恢复进程）
    pub const fn from_usize(pid: usize) -> Self {
        ProcessId(pid)
    }

    /// 获取PID的数值
    pub fn as_usize(self) -> usize {
        self.0
    }

    /// 检查是否为init进程（PID = 1）
    pub fn is_init(self) -> bool {
        self.0 == 1
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================
// 测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_pid_uniqueness() {
        let pid1 = ProcessId::new();
        let pid2 = ProcessId::new();
        assert_ne!(pid1, pid2);
    }

    #[test_case]
    fn test_pid_ordering() {
        let pid1 = ProcessId::new();
        let pid2 = ProcessId::new();
        assert!(pid1 < pid2);
    }
}
