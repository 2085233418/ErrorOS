#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::arch::global_asm;
use core::panic::PanicInfo;

// RISC-V 汇编入口点
global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "   la sp, stack_end",
    "   la t0, bss_start",
    "   la t1, bss_end",
    "1:",
    "   bgeu t0, t1, 2f",
    "   sd zero, (t0)",
    "   addi t0, t0, 8",
    "   j 1b",
    "2:",
    "   call test_main_entry",
    "3:",
    "   wfi",
    "   j 3b",
);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn test_main_entry() -> ! {
    use os::allocator;
    use os::memory;

    os::init();

    // 获取内核结束地址
    extern "C" {
        static kernel_end: u8;
    }
    let kernel_end_addr = unsafe { &kernel_end as *const u8 as usize };

    // 初始化内存管理
    let mut memory_manager = memory::init(kernel_end_addr);

    allocator::init_heap(&mut memory_manager.frame_allocator)
        .expect("heap initialization failed");

    test_main();
    loop {
        os::hlt_loop();
    }
}

use alloc::boxed::Box;
#[test_case]
fn simple_allocation(){
    let heap_value_1=Box::new(41);
    let heap_value_2=Box::new(13);
    assert_eq!(*heap_value_1,41);
    assert_eq!(*heap_value_2,13);
}

use alloc::vec::Vec;

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

use os::allocator::HEAP_SIZE;

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}
