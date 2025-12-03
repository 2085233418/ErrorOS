/*
 * ============================================
 * 兼容性模块：中断处理
 * ============================================
 *
 * ⚠️ 本模块已废弃，仅用于向后兼容
 *
 * 从第6章开始，中断和异常处理已迁移到 trap 模块。
 * 本文件仅重导出 trap 模块的功能，确保旧代码不会破坏。
 *
 * 新代码请使用：
 * ```rust
 * use crate::trap;
 *
 * trap::init();
 * trap::enable_interrupts();
 * ```
 *
 * 而不是：
 * ```rust
 * use crate::interrupts;
 *
 * interrupts::init_idt();
 * interrupts::enable_interrupts();
 * ```
 * ============================================
 */

// 重导出 trap 模块的所有公共项
pub use crate::trap::{
    init as init_idt,           // 兼容旧名称
    trap_handler,
    enable_interrupts,
    disable_interrupts,
    without_interrupts,
};
