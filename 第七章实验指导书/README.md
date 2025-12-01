# 第七章：文件系统 - 指导书目录

## 章节结构

### 7.1 虚拟文件系统接口

- **7.1 虚拟文件系统接口** ✅
  - 理解VFS设计哲学
  - 定义File trait接口
  - 实现Inode抽象
  - 设计文件描述符表
  - 实现标准输入输出

### 7.2 RamFS实现

- **7.2 简单文件系统实现** ✅
  - 设计RamInode结构
  - 实现文件读写
  - 实现目录操作
  - 实现RamFS文件系统
  - 创建全局管理器

### 7.3 文件系统系统调用

- **7.3 文件系统系统调用** ✅
  - 添加系统调用号
  - 实现sys_read
  - 实现sys_write
  - 实现sys_open/close
  - 实现sys_mkdir

### 7.4 文件系统测试

- **7.4 文件系统测试** ✅
  - 测试文件创建
  - 测试文件读写
  - 测试FD管理
  - 测试目录创建

---

## 当前进度

### 已完成 ✅

- **7.1 虚拟文件系统接口** - 全部完成
  - File trait定义 (read/write/seek)
  - Inode抽象和权限
  - 文件描述符表
  - 标准输入输出

- **7.2 RamFS实现** - 全部完成
  - RamInode实现
  - RamFile文件句柄
  - RamFS文件系统
  - 全局管理器

- **7.3 文件系统系统调用** - 全部完成
  - sys_read / sys_write
  - sys_open / sys_close
  - sys_mkdir
  - 系统调用分发

- **7.4 文件系统测试** - 全部完成
  - 文件创建测试
  - 读写测试
  - FD表测试
  - 目录测试

---

## 写作风格要求

1. **短代码段**：每个代码块10-30行
2. **HTML折叠**：完整代码用`<details>`折叠
3. **代码外讲解**：复杂算法在代码块外详细解释
4. **文件结构**：每节开头列出新增文件
5. **无可视化代码**：测试代码不包含可视化内容

---

## 核心概念

### VFS（虚拟文件系统）

- **File trait** - 统一的文件操作接口
- **Inode** - 文件元数据管理
- **FD Table** - 文件描述符管理
- **一切皆文件** - Unix设计哲学

### RamFS设计

- **内存存储** - 数据在Vec<u8>中
- **目录树** - BTreeMap管理子文件
- **Inode分配** - 原子递增分配
- **Arc<Mutex<>>** - 共享所有权

### 系统调用

- **open** - 打开或创建文件
- **read/write** - 读写数据
- **close** - 关闭文件描述符
- **mkdir** - 创建目录

---

## 文件结构

```
os/src/fs/
├── mod.rs          # 模块入口
├── file.rs         # File trait定义
├── inode.rs        # Inode抽象
├── fd_table.rs     # 文件描述符表
├── stdio.rs        # 标准输入输出
├── ramfs.rs        # RamFS实现
└── manager.rs      # 全局管理器

os/src/syscall/
├── mod.rs              # 系统调用分发（修改）
└── syscall_impl.rs     # 系统调用实现（修改）

os/tests/
└── test_filesystem.rs  # 文件系统测试
```

---

## 依赖关系

```
7.1 VFS接口
  ├─ File trait (read/write/seek)
  ├─ Inode抽象
  ├─ FD Table
  └─ stdio
    ↓
7.2 RamFS实现
  ├─ RamInode
  ├─ RamFile
  └─ RamFS
    ↓
7.3 系统调用
  ├─ sys_read/write
  ├─ sys_open/close
  └─ sys_mkdir
    ↓
7.4 测试验证
```

按顺序学习，每节独立可测试。

---

## 实现亮点

### 类型安全

```rust
pub trait File: Send + Sync {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError>;
}
```

使用trait object实现运行时多态。

### 零拷贝

```rust
let slice = unsafe { core::slice::from_raw_parts(buf, len) };
```

直接操作用户态缓冲区，避免拷贝。

### 线程安全

```rust
lazy_static! {
    pub static ref FD_TABLE: Mutex<FileDescriptorTable> = { ... };
}
```

全局FD表使用Mutex保护。

---

## 性能特点

| 操作 | 时间复杂度 | 说明 |
|------|-----------|------|
| open | O(log n) | BTreeMap查找 |
| read | O(n) | 内存拷贝 |
| write | O(n) | 内存拷贝 |
| close | O(1) | FD释放 |
| mkdir | O(log n) | BTreeMap插入 |

---

## 常见问题

**Q1: 为什么用RamFS而不是真实文件系统？**

- 实现简单，适合学习
- 无需处理磁盘I/O
- 性能更好，适合测试

**Q2: 为什么File和Inode分开？**

- Inode是文件本身（元数据）
- File是打开的句柄（读写位置）
- 一个文件可以被多次打开

**Q3: 为什么FD从3开始？**

- 0-2保留给stdin/stdout/stderr
- 兼容Unix标准

---

## 扩展方向

### 功能扩展

- [ ] 实现真实文件系统（FAT32/ext2）
- [ ] 支持文件权限检查
- [ ] 实现文件锁
- [ ] 添加缓冲区缓存

### 性能优化

- [ ] 实现页缓存
- [ ] 优化目录查找（哈希表）
- [ ] 异步I/O支持

### 安全增强

- [ ] 地址空间验证
- [ ] 路径遍历检查
- [ ] 配额限制

---

## 学习建议

1. **先理解VFS** - 掌握抽象层设计
2. **实现RamFS** - 理解文件系统内部
3. **集成系统调用** - 连接用户态和内核
4. **编写测试** - 验证功能正确性

---

## 下一步

第7章完成！建议继续：

- **第8章**：实现持久化存储（磁盘驱动）
- **第9章**：实现真实文件系统（FAT32）
- **第10章**：网络协议栈

---

## 参考资料

- [xv6文件系统](https://pdos.csail.mit.edu/6.828/2020/xv6/book-riscv-rev1.pdf)
- [Linux VFS文档](https://www.kernel.org/doc/html/latest/filesystems/vfs.html)
- [Rust std::fs](https://doc.rust-lang.org/std/fs/)
