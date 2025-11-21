/*
 * ============================================
 * RISC-V 键盘输入模块
 * ============================================
 * 功能：处理键盘输入（通过 SBI console）
 *
 * RISC-V 键盘输入方案：
 * - 使用 SBI (Supervisor Binary Interface) 的 console_getchar
 * - 轮询方式读取字符
 * - 支持异步任务
 * ============================================
 */

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::task::{Context, Poll};
use core::pin::Pin;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;

/// 扫描码队列（用于存储输入字符）
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// 唤醒器
static WAKER: AtomicWaker = AtomicWaker::new();

/// 添加字符到队列
///
/// # 功能
/// - 被输入处理器调用
/// - 不能阻塞或分配内存
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            // 队列满时静默丢弃，避免频繁输出
        } else {
            WAKER.wake(); // 唤醒等待的任务
        }
    }
    // 如果队列未初始化，静默忽略（在键盘任务启动前可能发生）
}

/// 扫描码流（实现 Stream trait）
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    /// 创建新的扫描码流
    pub fn new() -> Self {
        // 尝试初始化队列，如果已经初始化则忽略错误
        let _ = SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100));
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // 尝试从队列中读取
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // 注册唤醒器
        WAKER.register(cx.waker());

        // 再次检查（防止竞争条件）
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

/// SBI console getchar
///
/// # 返回
/// - Some(char): 读取到的字符
/// - None: 没有可用字符
fn sbi_console_getchar() -> Option<u8> {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "li a7, 2",      // SBI extension ID: Console Getchar (legacy)
            "ecall",
            "mv {}, a0",     // 返回值在 a0
            out(reg) ret,
            options(nostack)
        );
    }

    if ret >= 0 {
        Some(ret as u8)
    } else {
        None
    }
}

/// 轮询键盘输入
///
/// # 功能
/// - 定期调用以检查键盘输入
/// - 应该在定时器中断中调用
/// - 限制每次最多读取的字符数，防止阻塞
pub fn poll_keyboard() {
    // 限制每次中断最多读取 10 个字符，防止无限循环
    const MAX_READS_PER_POLL: usize = 10;

    for _ in 0..MAX_READS_PER_POLL {
        if let Some(ch) = sbi_console_getchar() {
            add_scancode(ch);
        } else {
            // 没有更多字符可读，退出
            break;
        }
    }
}

/// 异步键盘任务
///
/// # 功能
/// - 持续读取键盘输入并显示
pub async fn print_keypresses() {
    use futures_util::stream::StreamExt;

    crate::serial_println!("[KEYBOARD] Keyboard input task started (SBI console)");
    crate::println!("[KEYBOARD] Press keys to test...");

    let mut scancodes = ScancodeStream::new();

    while let Some(scancode) = scancodes.next().await {
        // 处理特殊字符
        match scancode {
            b'\r' | b'\n' => {
                crate::println!();
            }
            0x08 | 0x7f => {
                // Backspace
                crate::print!("\x08 \x08");
            }
            0x20..=0x7e => {
                // 可打印 ASCII 字符
                crate::print!("{}", scancode as char);
            }
            _ => {
                // 其他字符显示为十六进制
                crate::print!("[{:02x}]", scancode);
            }
        }
    }
}

/// 键盘输入循环（用于定时器中断）
///
/// # 功能
/// - 在定时器中断中调用此函数
/// - 轮询 SBI console 获取输入
pub fn keyboard_interrupt_handler() {
    poll_keyboard();
}
