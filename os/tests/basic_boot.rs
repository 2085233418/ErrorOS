#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::global_asm;
use core::panic::PanicInfo;
use os::{QemuExitCode, exit_qemu, hlt_loop, println};

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
    "   call test_kernel_main",
    "3:",
    "   wfi",
    "   j 3b",
);

#[no_mangle]
pub extern "C" fn test_kernel_main() -> ! {
    test_main();
    loop {
        hlt_loop();
    }
}

// 实现测试运行器：执行所有测试用例
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info);
    hlt_loop();
}

// 测试用例：验证 println 功能
#[test_case]
fn test_println() {
    println!("test_println output");
}
