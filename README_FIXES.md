# Blog OS - RISC-V 操作系统

## 问题修复说明

### 已修复的问题

1. **终端无限刷新问题**
   - 原因：`poll_keyboard()` 使用无限 while 循环，可能导致在中断处理中阻塞
   - 修复：限制每次中断最多读取 10 个字符，防止无限循环

2. **定时器中断过于频繁**
   - 原因：定时器间隔设置为 10ms，过于频繁
   - 修复：将间隔调整为 100ms，减少中断频率

3. **无法用快捷键退出 QEMU**
   - 原因：使用 `-nographic` 模式集成在终端中
   - 修复：提供 GUI 窗口模式和改进的终端模式

## 运行方式

### 方式 1：使用 Makefile（推荐）

```bash
# 构建操作系统
make build

# 在独立 GUI 窗口中运行（推荐）
make run-gui

# 在终端中运行（可用 Ctrl+A 然后 X 退出）
make run-console

# 查看帮助
make help

# 清理构建文件
make clean
```

### 方式 2：使用脚本

```bash
# GUI 窗口模式（推荐）
./run_gui.sh

# 终端模式
./run_console.sh
```

### 方式 3：使用 Cargo

```bash
cd os

# 仅构建（不自动运行）
cargo build

# 手动运行 QEMU
qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -smp 1 \
    -m 128M \
    -bios default \
    -kernel target/riscv64imac-unknown-none-elf/debug/os \
    -serial stdio \
    -display default
```

## QEMU 使用提示

### GUI 窗口模式
- 按 `Ctrl+Alt+G` 可以释放鼠标焦点
- 直接关闭窗口即可退出 QEMU
- 串口输出会显示在启动终端中

### 终端模式
- 按 `Ctrl+A` 然后按 `X` 退出 QEMU
- 或者按 `Ctrl+C` 强制退出
- 所有输出都在当前终端中

## 项目结构

```
Blog_OS/
├── os/                          # 操作系统源代码
│   ├── src/
│   │   ├── main.rs             # 内核入口
│   │   ├── interrupts.rs       # 中断处理
│   │   ├── task/
│   │   │   ├── keyboard.rs     # 键盘输入
│   │   │   └── executor.rs     # 异步执行器
│   │   └── ...
│   ├── .cargo/config.toml      # Cargo 配置
│   └── Cargo.toml
├── Makefile                     # 构建脚本
├── run_gui.sh                   # GUI 运行脚本
└── run_console.sh               # 终端运行脚本
```

## 技术细节

### 修改的文件

1. **os/.cargo/config.toml**
   - 注释掉了自动 runner 配置
   - 避免 `cargo run` 自动启动 QEMU

2. **os/src/interrupts.rs**
   - 将定时器中断间隔从 100,000 增加到 1,000,000 时钟周期
   - 降低中断频率，减少系统负载

3. **os/src/task/keyboard.rs**
   - 在 `poll_keyboard()` 中添加最大读取次数限制
   - 防止在中断处理中陷入无限循环

## 开发建议

- **推荐使用 GUI 模式**：更稳定，易于调试
- **终端模式**：适合在没有图形界面的环境中使用
- **调试**：可以在 QEMU 启动命令中添加 `-s -S` 参数使用 GDB 调试
