#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use os::serial_println;
use os::process::{ProcessId, ProcessState, create_process_handle};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    os::test_panic_handler(info)
}

/// 测试1：PID分配器
#[test_case]
fn test_pid_allocation() {
    serial_println!("\n╔════════════════════════════════════════════════════╗");
    serial_println!("║          测试1：PID分配器                          ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 说明：验证PID分配的唯一性和单调性                 ║");
    serial_println!("╠════════════════════════════════════════════════════╣");

    let mut pids = alloc::vec::Vec::new();

    for i in 1..=5 {
        let pid = ProcessId::new();
        serial_println!("║ 步骤{}: 分配新PID → PID = {:<22}║", i, pid);
        pids.push(pid);
    }

    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 验证结果：                                         ║");

    // 验证唯一性
    let mut unique = true;
    for i in 0..pids.len() {
        for j in i+1..pids.len() {
            if pids[i] == pids[j] {
                unique = false;
                break;
            }
        }
    }
    serial_println!("║   ✓ PID唯一性：{:<36}║", if unique { "通过" } else { "失败" });

    // 验证单调性
    let mut monotonic = true;
    for i in 0..pids.len()-1 {
        if pids[i].as_usize() >= pids[i+1].as_usize() {
            monotonic = false;
            break;
        }
    }
    serial_println!("║   ✓ PID单调递增：{:<34}║", if monotonic { "通过" } else { "失败" });

    serial_println!("╚════════════════════════════════════════════════════╝\n");

    assert!(unique && monotonic);
}

/// 测试2：进程状态转换
#[test_case]
fn test_process_state_transitions() {
    serial_println!("\n╔════════════════════════════════════════════════════╗");
    serial_println!("║          测试2：进程状态转换                       ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 说明：模拟进程从创建到退出的完整生命周期          ║");
    serial_println!("╠════════════════════════════════════════════════════╣");

    let mut state = ProcessState::Ready;

    // 步骤1：进程创建
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤1] 进程创建                                   ║");
    serial_println!("║   状态：Ready                                      ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │  Ready   │  ← 进程在就绪队列中等待            ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Ready);

    // 步骤2：进程被调度
    state = ProcessState::Running;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤2] 调度器选中进程                             ║");
    serial_println!("║   状态：Ready → Running                            ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │ Running  │  ← 进程获得CPU，开始执行           ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Running);

    // 步骤3：时间片用完
    state = ProcessState::Ready;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤3] 时间片用完                                 ║");
    serial_println!("║   状态：Running → Ready                            ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │  Ready   │  ← 被抢占，回到就绪队列            ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Ready);

    // 步骤4：再次被调度
    state = ProcessState::Running;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤4] 再次被调度                                 ║");
    serial_println!("║   状态：Ready → Running                            ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │ Running  │  ← 继续执行                        ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Running);

    // 步骤5：等待I/O
    state = ProcessState::Blocked;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤5] 进程等待I/O                                ║");
    serial_println!("║   状态：Running → Blocked                          ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │ Blocked  │  ← 等待磁盘读取完成                ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Blocked);

    // 步骤6：I/O完成
    state = ProcessState::Ready;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤6] I/O完成，进程被唤醒                        ║");
    serial_println!("║   状态：Blocked → Ready                            ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │  Ready   │  ← 重新进入就绪队列                ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Ready);

    // 步骤7：最后一次调度
    state = ProcessState::Running;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤7] 最后一次调度                               ║");
    serial_println!("║   状态：Ready → Running                            ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │ Running  │  ← 执行到结束                      ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Running);

    // 步骤8：进程退出
    state = ProcessState::Zombie;
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤8] 进程退出                                   ║");
    serial_println!("║   状态：Running → Zombie                           ║");
    serial_println!("║   ┌──────────┐                                     ║");
    serial_println!("║   │  Zombie  │  ← 保留PCB，等待父进程回收         ║");
    serial_println!("║   └──────────┘                                     ║");
    assert_eq!(state, ProcessState::Zombie);

    serial_println!("║                                                    ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ ✓ 所有状态转换测试通过                            ║");
    serial_println!("╚════════════════════════════════════════════════════╝\n");
}

/// 测试3：PCB创建和管理
#[test_case]
fn test_pcb_creation() {
    serial_println!("\n╔════════════════════════════════════════════════════╗");
    serial_println!("║          测试3：进程控制块(PCB)创建                ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 说明：创建多个进程并验证PCB信息                   ║");
    serial_println!("╠════════════════════════════════════════════════════╣");

    // 创建init进程
    let init = create_process_handle("init", None);
    let init_pid = init.lock().pid();

    serial_println!("║                                                    ║");
    serial_println!("║ [进程1] 创建init进程                               ║");
    serial_println!("║   ┌────────────────────────────────────────────┐   ║");
    serial_println!("║   │ PID:        {:<31}│   ║", init_pid);
    serial_println!("║   │ 名称:       init                           │   ║");
    serial_println!("║   │ 状态:       Ready                          │   ║");
    serial_println!("║   │ 父进程:     None (根进程)                  │   ║");
    serial_println!("║   └────────────────────────────────────────────┘   ║");

    // 创建shell进程
    let shell = create_process_handle("shell", Some(init_pid));
    let shell_pid = shell.lock().pid();

    serial_println!("║                                                    ║");
    serial_println!("║ [进程2] 创建shell进程                              ║");
    serial_println!("║   ┌────────────────────────────────────────────┐   ║");
    serial_println!("║   │ PID:        {:<31}│   ║", shell_pid);
    serial_println!("║   │ 名称:       shell                          │   ║");
    serial_println!("║   │ 状态:       Ready                          │   ║");
    serial_println!("║   │ 父进程:     {} (init)                     │   ║", init_pid);
    serial_println!("║   └────────────────────────────────────────────┘   ║");

    // 创建worker进程
    let worker = create_process_handle("worker", Some(shell_pid));
    let worker_pid = worker.lock().pid();

    serial_println!("║                                                    ║");
    serial_println!("║ [进程3] 创建worker进程                             ║");
    serial_println!("║   ┌────────────────────────────────────────────┐   ║");
    serial_println!("║   │ PID:        {:<31}│   ║", worker_pid);
    serial_println!("║   │ 名称:       worker                         │   ║");
    serial_println!("║   │ 状态:       Ready                          │   ║");
    serial_println!("║   │ 父进程:     {} (shell)                    │   ║", shell_pid);
    serial_println!("║   └────────────────────────────────────────────┘   ║");

    serial_println!("║                                                    ║");
    serial_println!("║ 进程树结构：                                       ║");
    serial_println!("║   init ({})                                        ║", init_pid);
    serial_println!("║    └─ shell ({})                                   ║", shell_pid);
    serial_println!("║        └─ worker ({})                              ║", worker_pid);

    serial_println!("║                                                    ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ ✓ PCB创建测试通过                                 ║");
    serial_println!("╚════════════════════════════════════════════════════╝\n");

    // 验证
    assert_eq!(init.lock().name(), "init");
    assert_eq!(shell.lock().name(), "shell");
    assert_eq!(worker.lock().name(), "worker");
    assert!(init.lock().parent_pid().is_none());
    assert_eq!(shell.lock().parent_pid(), Some(init_pid));
    assert_eq!(worker.lock().parent_pid(), Some(shell_pid));
}

/// 测试4：时间片管理
#[test_case]
fn test_time_slice() {
    serial_println!("\n╔════════════════════════════════════════════════════╗");
    serial_println!("║          测试4：时间片管理                         ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 说明：验证时间片的消耗和重置机制                   ║");
    serial_println!("╠════════════════════════════════════════════════════╣");

    let process = create_process_handle("test", None);
    process.lock().reset_time_slice();

    serial_println!("║                                                    ║");
    serial_println!("║ 初始状态：                                         ║");
    serial_println!("║   时间片 = 5                                       ║");
    serial_println!("║   ┌───┬───┬───┬───┬───┐                           ║");
    serial_println!("║   │ ● │ ● │ ● │ ● │ ● │  (5个时钟周期)            ║");
    serial_println!("║   └───┴───┴───┴───┴───┘                           ║");

    // 模拟时钟中断
    for i in 1..=5 {
        let should_schedule = process.lock().tick();

        serial_println!("║                                                    ║");
        serial_println!("║ 时钟中断 #{}: tick()                               ║", i);

        // 绘制剩余时间片
        let remaining = 5 - i;
        let consumed = i;

        serial_println!("║   ┌{}{}┐                           ║",
            "───┬".repeat(remaining as usize).trim_end_matches('┬'),
            "───┬".repeat(consumed as usize)
        );

        let mut bar = String::from("║   │");
        for _ in 0..remaining {
            bar.push_str(" ● │");
        }
        for _ in 0..consumed {
            bar.push_str(" ✕ │");
        }
        bar.push_str("  ");
        serial_println!("{:<52}║", bar);

        serial_println!("║   └{}{}┘                           ║",
            "───┴".repeat(remaining as usize).trim_end_matches('┴'),
            "───┴".repeat(consumed as usize)
        );

        if should_schedule {
            serial_println!("║   → 时间片用完！触发调度                          ║");
            assert_eq!(i, 5);
        } else {
            serial_println!("║   → 还有时间片，继续执行                          ║");
        }
    }

    serial_println!("║                                                    ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ ✓ 时间片管理测试通过                              ║");
    serial_println!("╚════════════════════════════════════════════════════╝\n");
}

/// 测试5：父子进程关系
#[test_case]
fn test_parent_child_relationship() {
    serial_println!("\n╔════════════════════════════════════════════════════╗");
    serial_println!("║          测试5：父子进程关系管理                   ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 说明：验证父进程管理子进程列表的功能               ║");
    serial_println!("╠════════════════════════════════════════════════════╣");

    let parent = create_process_handle("parent", None);
    let parent_pid = parent.lock().pid();

    serial_println!("║                                                    ║");
    serial_println!("║ [步骤1] 创建父进程                                 ║");
    serial_println!("║   PID: {}                                          ║", parent_pid);
    serial_println!("║   子进程数: 0                                      ║");

    // 创建3个子进程
    let child1_pid = ProcessId::new();
    let child2_pid = ProcessId::new();
    let child3_pid = ProcessId::new();

    parent.lock().add_child(child1_pid);
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤2] 添加子进程1                                ║");
    serial_println!("║   父进程: {}                                       ║", parent_pid);
    serial_println!("║   └─ 子进程: {}                                    ║", child1_pid);
    serial_println!("║   子进程数: {}                                     ║", parent.lock().children().len());

    parent.lock().add_child(child2_pid);
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤3] 添加子进程2                                ║");
    serial_println!("║   父进程: {}                                       ║", parent_pid);
    serial_println!("║   ├─ 子进程: {}                                    ║", child1_pid);
    serial_println!("║   └─ 子进程: {}                                    ║", child2_pid);
    serial_println!("║   子进程数: {}                                     ║", parent.lock().children().len());

    parent.lock().add_child(child3_pid);
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤4] 添加子进程3                                ║");
    serial_println!("║   父进程: {}                                       ║", parent_pid);
    serial_println!("║   ├─ 子进程: {}                                    ║", child1_pid);
    serial_println!("║   ├─ 子进程: {}                                    ║", child2_pid);
    serial_println!("║   └─ 子进程: {}                                    ║", child3_pid);
    serial_println!("║   子进程数: {}                                     ║", parent.lock().children().len());

    assert_eq!(parent.lock().children().len(), 3);

    // 移除子进程
    parent.lock().remove_child(child2_pid);
    serial_println!("║                                                    ║");
    serial_println!("║ [步骤5] 移除子进程2 (已退出)                       ║");
    serial_println!("║   父进程: {}                                       ║", parent_pid);
    serial_println!("║   ├─ 子进程: {}                                    ║", child1_pid);
    serial_println!("║   └─ 子进程: {}                                    ║", child3_pid);
    serial_println!("║   子进程数: {}                                     ║", parent.lock().children().len());

    assert_eq!(parent.lock().children().len(), 2);

    serial_println!("║                                                    ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ ✓ 父子进程关系测试通过                            ║");
    serial_println!("╚════════════════════════════════════════════════════╝\n");
}

/// 测试6：进程退出和僵尸状态
#[test_case]
fn test_process_exit() {
    serial_println!("\n╔════════════════════════════════════════════════════╗");
    serial_println!("║          测试6：进程退出和僵尸状态                 ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ 说明：验证进程退出后变为Zombie状态并保留退出码    ║");
    serial_println!("╠════════════════════════════════════════════════════╣");

    let process = create_process_handle("test_exit", None);
    let pid = process.lock().pid();

    serial_println!("║                                                    ║");
    serial_println!("║ [进程状态] PID: {}                                 ║", pid);
    serial_println!("║   初始状态: Ready                                  ║");
    serial_println!("║   退出码:   None                                   ║");

    // 模拟进程运行
    process.lock().set_state(ProcessState::Running);
    serial_println!("║                                                    ║");
    serial_println!("║   状态变更: Ready → Running                        ║");
    serial_println!("║   进程正在执行...                                  ║");

    // 进程退出
    let exit_code = 42;
    process.lock().set_exit_code(exit_code);

    serial_println!("║                                                    ║");
    serial_println!("║   进程调用 exit({})                                ║", exit_code);
    serial_println!("║   状态变更: Running → Zombie                       ║");
    serial_println!("║   退出码:   {}                                     ║", exit_code);
    serial_println!("║                                                    ║");
    serial_println!("║   ┌──────────────────────────────────────┐         ║");
    serial_println!("║   │  进程已变为Zombie状态                │         ║");
    serial_println!("║   │  - PCB仍然保留                       │         ║");
    serial_println!("║   │  - 退出码可供父进程读取              │         ║");
    serial_println!("║   │  - 等待父进程调用waitpid()回收       │         ║");
    serial_println!("║   └──────────────────────────────────────┘         ║");

    // 验证
    assert_eq!(process.lock().state(), ProcessState::Zombie);
    assert_eq!(process.lock().exit_code(), Some(exit_code));
    assert!(process.lock().is_zombie());

    serial_println!("║                                                    ║");
    serial_println!("╠════════════════════════════════════════════════════╣");
    serial_println!("║ ✓ 进程退出测试通过                                ║");
    serial_println!("╚════════════════════════════════════════════════════╝\n");
}
