#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::global_asm;
use core::panic::PanicInfo;
use os::println;
use os::task::keyboard;
use os::task::executor::Executor;

/// RISC-V 汇编入口点
///
/// 定义在汇编中，负责：
/// - 清零 BSS 段
/// - 设置栈指针
/// - 跳转到 kernel_main
global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    // 设置栈指针
    "   la sp, stack_end",
    // 清零 BSS 段
    "   la t0, bss_start",
    "   la t1, bss_end",
    "1:",
    "   bgeu t0, t1, 2f",
    "   sd zero, (t0)",
    "   addi t0, t0, 8",
    "   j 1b",
    "2:",
    // 跳转到 kernel_main
    "   call kernel_main",
    // 如果返回，进入死循环
    "3:",
    "   wfi",
    "   j 3b",
);
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();            // new
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}
extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use os::task::Task;

/// 内核主函数
///
/// # 功能
/// - 初始化内核
/// - 设置内存管理
/// - 启动异步执行器
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    use os::memory;
    use os::allocator;

    println!("Welcome to Error OS{}", "!");
    os::init();

    // 获取内核结束地址（由链接器定义）
    extern "C" {
        static kernel_end: u8;
    }
    let kernel_end_addr = unsafe { &kernel_end as *const u8 as usize };

    // 初始化内存管理
    let mut memory_manager = memory::init(kernel_end_addr);

    allocator::init_heap(&mut memory_manager.frame_allocator)
        .expect("heap initialization failed");

    let heap_value=Box::new(41);
    println!("heap_value {:p}",heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}",Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    // 启动异步执行器
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    println!("It did not crash!");
    os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}