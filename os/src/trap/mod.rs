/*
 * ============================================
 * RISC-V 陷阱（Trap）处理模块
 * ============================================
 * 功能：处理 RISC-V 中断（Interrupt）和异常（Exception）
 *
 * RISC-V 陷阱机制：
 * - stvec：陷阱向量基址寄存器
 * - scause：陷阱原因寄存器
 * - sepc：陷阱发生时的程序计数器
 * - stval：陷阱附加信息（如出错地址）
 * - sstatus：状态寄存器
 *
 * 支持的中断类型：
 * - 时钟中断（SupervisorTimer）
 * - 外部中断（SupervisorExternal）
 * - 软件中断（SupervisorSoft）
 *
 * 支持的异常类型：
 * - 系统调用（UserEnvCall）
 * - 页错误（Page Fault）
 * - 非法指令（Illegal Instruction）
 * - 断点（Breakpoint）
 * ============================================
 */

use crate::{serial_println, println};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sepc, stval, stvec,
};

/// 初始化陷阱处理系统
///
/// # 功能
/// - 设置 stvec 寄存器指向陷阱处理入口
/// - 启用定时器中断（用于进程调度）
/// - 设置第一个定时器中断
pub fn init() {
    unsafe {
        // 设置陷阱向量地址（Direct 模式）
        // 所有中断和异常都跳转到 trap_handler
        stvec::write(trap_handler as usize, stvec::TrapMode::Direct);
    }

    serial_println!("[INTERRUPT] Trap vector initialized");

    // 启用定时器中断
    unsafe {
        // 设置 sie 寄存器的 STIE 位（Supervisor Timer Interrupt Enable）
        riscv::register::sie::set_stimer();
    }

    // 设置第一次定时器中断
    set_next_timer();

    serial_println!("[INTERRUPT] Timer interrupt enabled");
}

/// 统一的陷阱处理入口
///
/// # 功能
/// - 读取 scause 寄存器判断陷阱类型
/// - 分发到对应的处理函数
///
/// # 调用约定
/// - 由硬件自动调用（通过 stvec 寄存器）
/// - 进入时硬件已自动保存部分上下文
#[no_mangle]
pub extern "C" fn trap_handler() {
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();

    match scause.cause() {
        // ============================================
        // 中断处理
        // ============================================
        Trap::Interrupt(interrupt) => {
            match interrupt {
                Interrupt::SupervisorTimer => {
                    timer_interrupt_handler();
                }
                Interrupt::SupervisorExternal => {
                    external_interrupt_handler();
                }
                Interrupt::SupervisorSoft => {
                    software_interrupt_handler();
                }
                _ => {
                    panic!(
                        "Unhandled interrupt!\n\
                        scause: {:?}\n\
                        sepc: {:#x}\n\
                        stval: {:#x}",
                        scause.cause(),
                        sepc,
                        stval
                    );
                }
            }
        }

        // ============================================
        // 异常处理
        // ============================================
        Trap::Exception(exception) => {
            match exception {
                Exception::Breakpoint => {
                    breakpoint_handler(sepc);
                }
                Exception::LoadPageFault |
                Exception::StorePageFault |
                Exception::InstructionPageFault => {
                    page_fault_handler(scause.cause(), stval, sepc);
                }
                Exception::IllegalInstruction => {
                    illegal_instruction_handler(sepc, stval);
                }
                Exception::UserEnvCall => {
                    // 系统调用处理入口
                    syscall_handler(sepc);
                }
                _ => {
                    panic!(
                        "Unhandled exception!\n\
                        scause: {:?}\n\
                        sepc: {:#x}\n\
                        stval: {:#x}",
                        scause.cause(),
                        sepc,
                        stval
                    );
                }
            }
        }
    }
}

// ============================================
// 中断处理函数
// ============================================

/// 时钟中断处理
///
/// # 功能
/// - 处理定时器中断
/// - 轮询键盘输入
/// - 设置下一次定时器中断
fn timer_interrupt_handler() {
    // 轮询键盘输入（通过 SBI console）
    crate::task::keyboard::poll_keyboard();

    // 设置下一次定时器中断
    set_next_timer();
}

/// 外部中断处理
///
/// # 功能
/// - 处理外部设备中断（如 UART、网卡等）
/// - 通过 PLIC（Platform-Level Interrupt Controller）管理
fn external_interrupt_handler() {
    serial_println!("[INTERRUPT] External interrupt received");
}

/// 软件中断处理
///
/// # 功能
/// - 处理核间中断（IPI, Inter-Processor Interrupt）
/// - 用于多核同步
fn software_interrupt_handler() {
    serial_println!("[INTERRUPT] Software interrupt received");
}

// ============================================
// 异常处理函数
// ============================================

/// 断点异常处理
///
/// # 参数
/// - `sepc`: 异常发生时的程序计数器
///
/// # 功能
/// - 处理 ebreak 指令触发的断点异常
/// - 用于调试
fn breakpoint_handler(sepc: usize) {
    serial_println!("[EXCEPTION] Breakpoint at {:#x}", sepc);
    println!("EXCEPTION: BREAKPOINT at {:#x}", sepc);

    // 断点指令后继续执行（跳过 ebreak 指令）
    riscv::register::sepc::write(sepc + 2); // ebreak 是 2 字节压缩指令
}

/// 页错误处理
///
/// # 参数
/// - `cause`: 异常类型（Load/Store/Instruction Page Fault）
/// - `stval`: 触发异常的虚拟地址
/// - `sepc`: 异常发生时的程序计数器
///
/// # 功能
/// - 处理访问无效内存地址的异常
/// - 未来可扩展为按需分页（Demand Paging）
fn page_fault_handler(cause: Trap, stval: usize, sepc: usize) {
    serial_println!(
        "[EXCEPTION] Page Fault\n\
        Type: {:?}\n\
        Address: {:#x}\n\
        PC: {:#x}",
        cause,
        stval,
        sepc
    );

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:#x}", stval);
    println!("Exception PC: {:#x}", sepc);
    println!("Fault Type: {:?}", cause);

    crate::hlt_loop();
}

/// 非法指令处理
///
/// # 参数
/// - `sepc`: 异常发生时的程序计数器
/// - `stval`: 非法指令的值
///
/// # 功能
/// - 处理执行非法指令的异常
fn illegal_instruction_handler(sepc: usize, stval: usize) {
    panic!(
        "EXCEPTION: ILLEGAL INSTRUCTION\n\
        PC: {:#x}\n\
        Instruction: {:#x}",
        sepc,
        stval
    );
}

/// 系统调用处理
///
/// # 参数
/// - `sepc`: 系统调用发生时的程序计数器
///
/// # 功能
/// - 处理用户态程序通过 ecall 指令触发的系统调用
/// - 系统调用号和参数通过寄存器传递：
///   - a7: 系统调用号
///   - a0-a5: 参数
///   - a0: 返回值
fn syscall_handler(sepc: usize) {
    // 从寄存器读取系统调用上下文
    let context = unsafe { crate::syscall::SyscallContext::from_registers() };

    // 调用系统调用分发器
    let result = crate::syscall::syscall_dispatcher(&context);

    // 设置返回值到 a0 寄存器
    unsafe {
        context.set_return_value(result);
    }

    // 系统调用返回后需要跳过 ecall 指令
    riscv::register::sepc::write(sepc + 4); // ecall 是 4 字节指令
}

// ============================================
// 中断控制函数
// ============================================

/// 禁用中断并执行闭包
///
/// # 功能
/// - 保存当前中断状态
/// - 禁用中断
/// - 执行闭包
/// - 恢复原始中断状态
///
/// # 用途
/// 用于实现临界区，防止死锁
///
/// # 示例
/// ```rust
/// let result = trap::without_interrupts(|| {
///     // 临界区代码
///     dangerous_operation()
/// });
/// ```
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use riscv::register::sstatus;

    // 读取当前中断状态
    let sie = sstatus::read().sie();

    if sie {
        // 如果中断启用，则禁用
        unsafe { riscv::register::sstatus::clear_sie(); }
    }

    // 执行闭包
    let ret = f();

    if sie {
        // 恢复中断状态
        unsafe { riscv::register::sstatus::set_sie(); }
    }

    ret
}

/// 启用中断
///
/// # 功能
/// - 设置 sstatus.SIE 位，允许中断发生
pub fn enable_interrupts() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}

/// 禁用中断
///
/// # 功能
/// - 清除 sstatus.SIE 位，禁止中断发生
pub fn disable_interrupts() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}

// ============================================
// 定时器相关
// ============================================

/// 设置下一次定时器中断
///
/// # 功能
/// - 通过 SBI 调用设置定时器
/// - 时间间隔：1,000,000 时钟周期（约 100ms @ 10MHz）
fn set_next_timer() {
    // QEMU RISC-V virt 机器的时钟频率为 10MHz
    const TIMER_INTERVAL: u64 = 1_000_000; // 100ms

    // 读取当前时间
    let time = riscv::register::time::read64();

    // 设置下一次定时器中断
    sbi_set_timer(time + TIMER_INTERVAL);
}

/// SBI 调用：设置定时器
///
/// # 参数
/// - `stime_value`: 定时器触发的时间值
///
/// # SBI 规范
/// - Extension ID (EID): 0x54494D45 ("TIME")
/// - Function ID (FID): 0 (SET_TIMER)
/// - 参数 a0: stime_value
fn sbi_set_timer(stime_value: u64) {
    unsafe {
        core::arch::asm!(
            "mv a0, {0}",         // 参数：时间值
            "li a7, 0",           // SBI extension ID: Timer (legacy)
            "ecall",              // 调用 SBI
            in(reg) stime_value,
            out("a0") _,
            out("a1") _,
            options(nostack)
        );
    }
}

// ============================================
// 测试
// ============================================

#[cfg(test)]
#[test_case]
fn test_breakpoint_exception() {
    use crate::serial_println;
    serial_println!("[TEST] test_breakpoint_exception...");

    // 触发断点异常
    unsafe {
        core::arch::asm!("ebreak");
    }

    serial_println!("[TEST] Breakpoint handled successfully");
}
