# RISC-V 操作系统

一个基于 Rust 的 RISC-V 64 位操作系统内核，移植自 x86_64 Blog OS 项目。

## 项目概述

本项目是一个教学型操作系统内核，实现了以下核心功能：

- ✅ **RISC-V 64 架构支持** - 完整的 RISC-V 64 位支持
- ✅ **中断和异常处理** - 支持断点、页错误、非法指令等异常
- ✅ **内存管理** - Sv39 分页机制，物理帧分配
- ✅ **堆分配器** - 固定大小块分配器，支持动态内存分配
- ✅ **异步任务系统** - 基于 Rust async/await 的协作式调度
- ✅ **串口输出** - UART 16550 驱动
- ✅ **系统调用接口** - 预留系统调用处理框架

## 架构特性

### RISC-V 特定实现

- **特权级别**: S-mode (Supervisor Mode)
- **分页机制**: Sv39 (39-bit virtual address)
- **页大小**: 4KB
- **中断控制**: PLIC (Platform-Level Interrupt Controller)
- **异常向量**: stvec 寄存器配置

### 内存布局

```
物理内存 (QEMU virt 机器):
  0x80000000 - 0x80100000   内核代码段
  0x80100000 - 0x80400000   内核数据和栈
  0x80400000 - 0x80500000   堆区域 (1 MB)
  0x80500000 - 0x88000000   可用物理内存

虚拟地址空间:
  使用恒等映射 (identity mapping)
  虚拟地址 = 物理地址
```

## 系统需求

### 必需工具

1. **Rust Toolchain** (nightly)
   ```bash
   rustup default nightly
   rustup component add rust-src llvm-tools-preview
   ```

2. **QEMU RISC-V**
   ```bash
   # macOS
   brew install qemu

   # Ubuntu/Debian
   sudo apt-get install qemu-system-riscv64

   # Arch Linux
   sudo pacman -S qemu-system-riscv
   ```

3. **RISC-V 工具链** (可选，用于调试)
   ```bash
   # macOS
   brew tap riscv/riscv
   brew install riscv-tools

   # Ubuntu/Debian
   sudo apt-get install gcc-riscv64-unknown-elf
   ```

## 编译和运行

### 1. 克隆项目

```bash
git clone <repository-url>
cd os
```

### 2. 编译内核

```bash
# 编译 (debug 模式)
cargo build

# 编译 (release 模式)
cargo build --release
```

编译产物位于 `target/riscv64gc-unknown-none-elf/debug/os` 或 `release/os`

### 3. 运行内核

```bash
# 直接运行 (使用 cargo run)
cargo run

# 或手动运行 QEMU
qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -smp 1 \
    -m 128M \
    -nographic \
    -serial mon:stdio \
    -bios none \
    -kernel target/riscv64gc-unknown-none-elf/debug/os
```

### 4. 退出 QEMU

按 `Ctrl-A` 然后按 `X`，或者运行 `Ctrl-C`

## 项目结构

```
os/
├── src/
│   ├── main.rs              # 内核入口点
│   ├── lib.rs               # 库入口
│   ├── console.rs           # 控制台输出
│   ├── serial.rs            # 串口驱动 (UART 16550)
│   ├── interrupts.rs        # 中断和异常处理
│   ├── memory.rs            # 内存管理
│   ├── allocator.rs         # 堆分配器
│   │   ├── bump.rs          # 碰撞分配器
│   │   ├── linked_list.rs   # 链表分配器
│   │   └── fixed_size_block.rs  # 固定大小块分配器
│   └── task/                # 异步任务系统
│       ├── mod.rs           # 任务抽象
│       ├── executor.rs      # 任务执行器
│       ├── simple_executor.rs  # 简单执行器
│       └── keyboard.rs      # 键盘任务 (待适配)
├── Cargo.toml               # 项目配置
├── linker-riscv64.ld        # RISC-V 链接脚本
├── riscv64gc-unknown-none-elf.json  # 自定义目标配置
├── .cargo/
│   └── config.toml          # Cargo 构建配置
└── README.md                # 本文件
```

## 核心模块说明

### 1. 启动流程 (`main.rs`)

```rust
_start (汇编入口)
  ↓
清零 BSS 段
  ↓
kernel_main (Rust 主函数)
  ↓
初始化中断系统
  ↓
初始化内存管理
  ↓
初始化堆分配器
  ↓
运行异步执行器
```

### 2. 中断处理 (`interrupts.rs`)

支持的中断和异常：

| 类型 | 名称 | 处理函数 |
|------|------|----------|
| 中断 | 时钟中断 | `timer_interrupt_handler` |
| 中断 | 外部中断 | `external_interrupt_handler` |
| 中断 | 软件中断 | `software_interrupt_handler` |
| 异常 | 断点 | `breakpoint_handler` |
| 异常 | 页错误 | `page_fault_handler` |
| 异常 | 非法指令 | `illegal_instruction_handler` |
| 异常 | 系统调用 | `syscall_handler` (预留) |

### 3. 内存管理 (`memory.rs`)

- **Sv39 分页**: 3 级页表，39 位虚拟地址
- **物理帧分配器**: 简单的 bump 分配器
- **页表管理**: 页表项操作和地址转换

### 4. 堆分配器 (`allocator/`)

使用**固定大小块分配器**：

- 支持的块大小: 8, 16, 32, 64, 128, 256, 512, 1024, 2048 字节
- 优点: 分配速度快 (O(1))，碎片化可控
- 后备分配器: `linked_list_allocator` 处理超大分配

### 5. 异步任务系统 (`task/`)

- **协作式调度**: 基于 Rust async/await
- **唤醒机制**: 通过 Waker 唤醒就绪任务
- **执行器**: 单线程执行器，使用优先队列

## 开发指南

### 添加新的系统调用

1. 在 `interrupts.rs` 的 `syscall_handler` 中添加处理逻辑：

```rust
fn syscall_handler(sepc: usize) {
    // 读取 a7 寄存器获取系统调用号
    let syscall_number = /* ... */;

    match syscall_number {
        1 => sys_write(/* args */),
        2 => sys_read(/* args */),
        // ...
    }
}
```

2. 实现系统调用函数：

```rust
fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // 实现写操作
}
```

### 添加新的异步任务

```rust
async fn my_task() {
    println!("Task started");
    // 异步操作
    println!("Task completed");
}

// 在 main.rs 中生成任务
executor.spawn(Task::new(my_task()));
```

### 调试技巧

1. **使用 GDB 调试**:

```bash
# 终端 1: 启动 QEMU (等待 GDB 连接)
qemu-system-riscv64 \
    -machine virt -cpu rv64 -smp 1 -m 128M \
    -nographic -serial mon:stdio -bios none \
    -kernel target/riscv64gc-unknown-none-elf/debug/os \
    -s -S

# 终端 2: 启动 GDB
riscv64-unknown-elf-gdb target/riscv64gc-unknown-none-elf/debug/os
(gdb) target remote localhost:1234
(gdb) break kernel_main
(gdb) continue
```

2. **查看串口输出**:

所有 `serial_println!` 和 `println!` 的输出都会显示在终端

3. **查看寄存器状态**:

在异常处理函数中添加：

```rust
use riscv::register::{sstatus, sepc, stval, scause};
println!("sstatus: {:?}", sstatus::read());
println!("sepc: {:#x}", sepc::read());
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test heap_allocation
```

## 性能优化

1. **编译优化**:

```toml
# Cargo.toml
[profile.release]
panic = "abort"
opt-level = 3
lto = true
```

2. **使用 release 模式**:

```bash
cargo build --release
cargo run --release
```

## 已知限制

1. **单核支持**: 当前仅支持单核 CPU
2. **无文件系统**: 暂未实现文件系统
3. **无网络栈**: 暂未实现网络功能
4. **有限的设备驱动**: 仅支持串口输出

## 未来计划

- [ ] 多核 SMP 支持
- [ ] 虚拟文件系统 (VFS)
- [ ] 用户态进程
- [ ] 进程间通信 (IPC)
- [ ] 网络协议栈
- [ ] 块设备驱动
- [ ] PLIC 完整支持
- [ ] SBI 调用封装

## 依赖说明

| Crate | 版本 | 用途 |
|-------|------|------|
| `riscv` | 0.11 | RISC-V 架构支持 |
| `uart_16550` | 0.3.0 | UART 串口驱动 |
| `spin` | 0.5.2 | 自旋锁 |
| `lazy_static` | 1.0 | 静态变量延迟初始化 |
| `linked_list_allocator` | 0.10.5 | 链表分配器 |
| `crossbeam-queue` | 0.3.11 | 无锁队列 |
| `futures-util` | 0.3.4 | 异步工具 |

## 致谢

本项目基于 [Writing an OS in Rust](https://os.phil-opp.com/) 教程，并移植到 RISC-V 架构。

## 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。

## 联系方式

如有问题或建议，请提交 Issue 或 Pull Request。

---

**Happy Hacking! 🚀**
