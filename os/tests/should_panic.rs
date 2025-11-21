#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]  // 引用当前测试 crate 的 test_runner
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
// 替换为你的主 crate 名称（Cargo.toml 中的 name = "os"）
use os::{QemuExitCode, exit_qemu, serial_println, serial_print};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");  // 测试预期会 panic，因此 panic 时视为成功
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[unsafe(no_mangle)]  // 移除 unsafe，no_mangle 不需要 unsafe 修饰
pub extern "C" fn _start() -> ! {
    test_main();  // 启动测试
    loop {}
}

// 测试运行器：如果测试未 panic，则视为失败
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();  // 执行测试用例（预期会 panic）
        // 如果测试没 panic，会执行到这里，标记为失败
        serial_println!("[test did not panic]");
        exit_qemu(QemuExitCode::Failed);
    }
    exit_qemu(QemuExitCode::Success);
}

// 预期会 panic 的测试用例
#[test_case]
fn should_fail() {
    serial_print!("should_fail... ");
    assert_eq!(0, 1);  // 必然触发 panic，测试成功
}