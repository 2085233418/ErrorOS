/*
 * ============================================
 * 进程上下文（Process Context）
 * ============================================
 * 功能：保存和恢复进程执行状态
 *
 * RISC-V 寄存器说明：
 * - x0 (zero): 硬编码为0，不需要保存
 * - x1 (ra): 返回地址
 * - x2 (sp): 栈指针
 * - x3 (gp): 全局指针
 * - x4 (tp): 线程指针
 * - x5-x7, x28-x31 (t0-t6): 临时寄存器
 * - x8-x9, x18-x27 (s0-s11): 保存寄存器
 * - x10-x17 (a0-a7): 参数/返回值寄存器
 *
 * 特殊CSR寄存器：
 * - sepc: 异常程序计数器（保存系统调用/中断时的PC）
 * - sstatus: 状态寄存器（保存特权级等状态）
 * - satp: 地址转换和保护寄存器（页表基址）
 * ============================================
 */

/// 进程上下文结构
///
/// # 内存布局
/// 该结构体使用 #[repr(C)] 确保字段顺序与汇编代码一致
/// 在上下文切换汇编中，我们按此顺序保存/恢复寄存器
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcessContext {
    // ============================================
    // 通用寄存器（按 RISC-V ABI 顺序）
    // ============================================

    /// x1: 返回地址 (Return Address)
    pub ra: usize,

    /// x2: 栈指针 (Stack Pointer)
    pub sp: usize,

    /// x3: 全局指针 (Global Pointer)
    pub gp: usize,

    /// x4: 线程指针 (Thread Pointer)
    pub tp: usize,

    /// x5-x7, x28-x31: 临时寄存器 (Temporary)
    /// 在函数调用时不需要保存，但在上下文切换时需要
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,

    /// x8-x9, x18-x27: 保存寄存器 (Saved)
    /// 函数调用需要保存这些寄存器
    pub s0: usize,  // fp (Frame Pointer)
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    /// x10-x17: 参数/返回值寄存器 (Arguments)
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,

    // ============================================
    // 特殊CSR寄存器
    // ============================================

    /// 异常程序计数器 (Supervisor Exception Program Counter)
    /// 保存异常/中断发生时的PC，用于返回
    pub sepc: usize,

    /// 状态寄存器 (Supervisor Status Register)
    /// 包含：
    /// - SPP: 异常发生前的特权级 (0=User, 1=Supervisor)
    /// - SPIE: 异常发生前的中断使能状态
    /// - SIE: 当前中断使能状态
    pub sstatus: usize,

    /// 地址转换寄存器 (Supervisor Address Translation and Protection)
    /// 格式（Sv39）：
    /// - MODE[63:60]: 分页模式（8=Sv39）
    /// - ASID[59:44]: 地址空间ID
    /// - PPN[43:0]: 页表物理页号
    pub satp: usize,
}

impl ProcessContext {
    /// 创建一个空的上下文
    ///
    /// # 说明
    /// 所有寄存器初始化为0
    /// 在实际使用前需要设置：
    /// - sp: 栈指针
    /// - sepc: 程序入口点
    /// - sstatus: 状态寄存器
    /// - satp: 页表基址
    pub const fn new() -> Self {
        ProcessContext {
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            sepc: 0,
            sstatus: 0,
            satp: 0,
        }
    }

    /// 为用户态进程初始化上下文
    ///
    /// # 参数
    /// - `entry_point`: 程序入口地址
    /// - `user_stack_top`: 用户栈顶地址
    /// - `satp_value`: 页表地址转换寄存器值
    ///
    /// # 说明
    /// 设置上下文使得进程：
    /// - 从 entry_point 开始执行
    /// - 使用 user_stack_top 作为栈
    /// - 运行在用户态（SPP=0）
    /// - 启用中断（SPIE=1）
    pub fn new_user_context(
        entry_point: usize,
        user_stack_top: usize,
        satp_value: usize,
    ) -> Self {
        let mut context = Self::new();

        // 设置程序入口点
        context.sepc = entry_point;

        // 设置用户栈
        context.sp = user_stack_top;

        // 设置页表
        context.satp = satp_value;

        // 设置 sstatus：
        // - SPP = User (0): 返回到用户态
        // - SPIE = 1: 返回后启用中断
        // 直接读取 CSR 寄存器
        let mut status_val: usize;
        unsafe {
            core::arch::asm!("csrr {}, sstatus", out(reg) status_val);
        }
        sstatus_ext::set_user_mode(&mut status_val);
        sstatus_ext::enable_interrupt_on_return(&mut status_val);
        context.sstatus = status_val;

        context
    }

    /// 零值初始化（用于测试）
    pub fn zero() -> Self {
        Self::new()
    }
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// 辅助结构体扩展
// ============================================

/// RISC-V sstatus 寄存器位域扩展
///
/// 由于 riscv crate 的限制，我们需要手动操作一些位
mod sstatus_ext {
    pub const SPP_BIT: usize = 8;   // 特权级位
    pub const SPIE_BIT: usize = 5;  // 中断使能位
    pub const SIE_BIT: usize = 1;   // 当前中断使能

    /// 设置为用户态
    pub fn set_user_mode(sstatus: &mut usize) {
        *sstatus &= !(1 << SPP_BIT);  // SPP = 0
    }

    /// 设置为内核态
    pub fn set_supervisor_mode(sstatus: &mut usize) {
        *sstatus |= 1 << SPP_BIT;  // SPP = 1
    }

    /// 启用中断（返回后）
    pub fn enable_interrupt_on_return(sstatus: &mut usize) {
        *sstatus |= 1 << SPIE_BIT;  // SPIE = 1
    }
}

// ============================================
// 上下文切换函数（外部汇编实现）
// ============================================

extern "C" {
    /// 上下文切换函数（在 switch.S 中实现）
    ///
    /// # 参数
    /// - `current_context`: 当前进程上下文的地址
    /// - `next_context`: 下一个进程上下文的地址
    ///
    /// # 说明
    /// 该函数：
    /// 1. 保存当前进程的所有寄存器到 current_context
    /// 2. 从 next_context 恢复下一个进程的所有寄存器
    /// 3. 切换页表（satp）
    /// 4. 返回到新进程继续执行
    ///
    /// 注意：此函数永不返回到原调用点
    pub fn switch_context(current_context: *mut ProcessContext, next_context: *const ProcessContext);
}

// ============================================
// 测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_context_size() {
        // 确保上下文大小合理（约34个usize）
        use core::mem::size_of;
        let size = size_of::<ProcessContext>();
        assert_eq!(size, 34 * size_of::<usize>());
    }

    #[test_case]
    fn test_context_new() {
        let ctx = ProcessContext::new();
        assert_eq!(ctx.sp, 0);
        assert_eq!(ctx.sepc, 0);
    }

    #[test_case]
    fn test_user_context_creation() {
        let entry = 0x1000_0000;
        let stack = 0x2000_0000;
        let satp = 0x8000_0000_0000_0000;

        let ctx = ProcessContext::new_user_context(entry, stack, satp);

        assert_eq!(ctx.sepc, entry);
        assert_eq!(ctx.sp, stack);
        assert_eq!(ctx.satp, satp);
    }
}
