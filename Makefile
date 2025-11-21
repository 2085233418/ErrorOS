# ============================================
# Blog OS Makefile
# ============================================

.PHONY: build run run-gui run-console clean help

# 默认目标
all: build

# 构建操作系统
build:
	@echo "正在构建操作系统..."
	@cd os && cargo build

# 在独立 GUI 窗口中运行（推荐）
run-gui: build
	@echo "启动 QEMU (GUI 窗口模式)..."
	@echo "提示: 按 Ctrl+Alt+G 释放鼠标，关闭窗口退出"
	@qemu-system-riscv64 \
		-machine virt \
		-cpu rv64 \
		-smp 1 \
		-m 128M \
		-bios default \
		-kernel os/target/riscv64imac-unknown-none-elf/debug/os \
		-serial stdio \
		-display default

# 在终端中运行（可用 Ctrl+C 退出）
run-console: build
	@echo "启动 QEMU (终端模式)..."
	@echo "提示: 按 Ctrl+A 然后按 X 退出，或按 Ctrl+C"
	@qemu-system-riscv64 \
		-machine virt \
		-cpu rv64 \
		-smp 1 \
		-m 128M \
		-nographic \
		-serial mon:stdio \
		-bios default \
		-kernel os/target/riscv64imac-unknown-none-elf/debug/os

# 默认使用 GUI 模式
run: run-gui

# 清理构建文件
clean:
	@echo "清理构建文件..."
	@cd os && cargo clean

# 显示帮助信息
help:
	@echo "Blog OS 构建系统"
	@echo ""
	@echo "可用命令:"
	@echo "  make build       - 构建操作系统"
	@echo "  make run         - 在 GUI 窗口中运行 (默认)"
	@echo "  make run-gui     - 在 GUI 窗口中运行 (推荐)"
	@echo "  make run-console - 在终端中运行 (Ctrl+A X 退出)"
	@echo "  make clean       - 清理构建文件"
	@echo "  make help        - 显示此帮助信息"
	@echo ""
	@echo "提示:"
	@echo "  - GUI 模式: 独立窗口，易于使用"
	@echo "  - 终端模式: 在当前终端中运行，按 Ctrl+A 然后 X 退出"
