/*
 * ============================================
 * RISC-V 串口驱动模块
 * ============================================
 * 功能：提供 UART 16550 串口输出功能
 * 用途：调试输出、日志记录、与 QEMU 通信
 *
 * RISC-V QEMU virt 机器的串口地址：0x10000000
 * ============================================
 */

use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
use volatile::Volatile;

// RISC-V QEMU virt 机器的 UART 基地址
const UART_BASE_ADDRESS: usize = 0x1000_0000;

/// UART 16550 寄存器偏移
const UART_THR: usize = 0; // Transmitter Holding Register
const UART_LSR: usize = 5; // Line Status Register

/// Line Status Register 位定义
const UART_LSR_THRE: u8 = 1 << 5; // Transmitter Holding Register Empty

/// 简单的 UART 串口驱动
pub struct SerialPort {
    base_address: usize,
}

impl SerialPort {
    /// 创建新的串口实例
    pub unsafe fn new(base_address: usize) -> Self {
        SerialPort { base_address }
    }

    /// 初始化串口
    pub fn init(&mut self) {
        // QEMU 的 UART 默认已初始化，无需额外配置
    }

    /// 发送一个字节
    fn send(&mut self, byte: u8) {
        unsafe {
            // 等待发送缓冲区为空
            while !self.is_transmit_empty() {}

            // 写入数据
            let thr = (self.base_address + UART_THR) as *mut Volatile<u8>;
            (*thr).write(byte);
        }
    }

    /// 检查发送缓冲区是否为空
    fn is_transmit_empty(&self) -> bool {
        unsafe {
            let lsr = (self.base_address + UART_LSR) as *const Volatile<u8>;
            (*lsr).read() & UART_LSR_THRE != 0
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

lazy_static! {
    /// 全局串口实例（UART0）
    ///
    /// 使用 Mutex 保护以支持多核访问
    /// 在 RISC-V QEMU virt 机器中，UART 映射到 0x10000000
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(UART_BASE_ADDRESS) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// 底层打印函数
///
/// # 功能
/// - 格式化输出到串口
/// - 在临界区内执行，禁用中断以防止死锁
///
/// # 参数
/// - `args`: 格式化参数
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    // RISC-V 中断禁用/启用
    // 使用自旋锁时禁用中断，防止死锁
    crate::interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// 串口打印宏
///
/// # 用法
/// ```rust
/// serial_print!("Hello, RISC-V!");
/// serial_print!("Value: {}", 42);
/// ```
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// 串口打印宏（带换行）
///
/// # 用法
/// ```rust
/// serial_println!();                    // 仅换行
/// serial_println!("Hello, RISC-V!");    // 打印并换行
/// serial_println!("x = {}", x);         // 格式化打印
/// ```
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
