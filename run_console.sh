#!/bin/bash
# ============================================
# QEMU 终端模式运行脚本（可用 Ctrl+C 退出）
# ============================================

set -e

echo "正在构建操作系统..."
cd os
cargo build

echo "启动 QEMU（终端模式）..."
echo "提示："
echo "  - 按 Ctrl+C 可以退出 QEMU"
echo "  - 或者按 Ctrl+A 然后按 X 退出"
echo ""

# 使用 -serial mon:stdio 但添加信号处理
# stty -echo 防止回显干扰
qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -smp 1 \
    -m 128M \
    -nographic \
    -serial mon:stdio \
    -bios default \
    -kernel target/riscv64imac-unknown-none-elf/debug/os
