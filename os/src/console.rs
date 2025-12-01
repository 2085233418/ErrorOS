/*
 * ============================================
 * RISC-V 控制台输出模块
 * ============================================
 * 功能：提供控制台输出功能（替代 原内核x86架构中VGA 缓冲区）
 * 实现：通过串口输出（RISC-V 没有 VGA 设备）
 *
 * 在 RISC-V 环境中，我们使用串口作为主要的输出设备
 * ============================================
 */

use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// 全局 Writer 实例
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

/// 控制台写入器
pub struct Writer {
    column_position: usize,
}

impl Writer {
    /// 创建新的 Writer
    pub const fn new() -> Self {
        Writer {
            column_position: 0,
        }
    }

    /// 写入字节
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            }
            byte => {
                // 通过串口输出
                self.write_to_serial(byte);
                self.column_position += 1;
            }
        }
    }

    /// 写入字符串
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可打印 ASCII 字符或换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不可打印字符，输出 ■
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// 换行
    fn new_line(&mut self) {
        self.write_to_serial(b'\n');
        self.column_position = 0;
    }

    /// 通过串口输出字节
    fn write_to_serial(&mut self, byte: u8) {
        use crate::serial::SERIAL1;
        use core::fmt::Write;

        // 直接写入串口（不需要通过临界区，因为已经持有 WRITER 锁）
        let mut serial = SERIAL1.lock();
        let _ = serial.write_char(byte as char);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// 底层打印函数
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use crate::interrupts;

    // 在临界区内执行，禁用中断以防止死锁
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// 打印宏（不换行）
///
/// # 用法
/// ```rust
/// print!("Hello");
/// print!("x = {}", x);
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

/// 打印宏（换行）
///
/// # 用法
/// ```rust
/// println!();                   // 仅换行
/// println!("Hello, RISC-V!");   // 打印并换行
/// println!("x = {}", x);        // 格式化打印
/// ```
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
