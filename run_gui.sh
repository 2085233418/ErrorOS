#!/bin/bash
# ============================================
# QEMU GUI 模式运行脚本（独立窗口）
# ============================================

set -e

echo "正在构建操作系统..."
cd os
cargo build

echo "启动 QEMU（图形窗口模式）..."
echo "提示："
echo "  - QEMU 将在独立窗口中打开"
echo "  - 在 QEMU 窗口中按 Ctrl+Alt+G 可以释放鼠标"
echo "  - 关闭窗口即可退出 QEMU"
echo ""

qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -smp 1 \
    -m 128M \
    -bios default \
    -kernel target/riscv64imac-unknown-none-elf/debug/os \
    -serial stdio \
    -display default
