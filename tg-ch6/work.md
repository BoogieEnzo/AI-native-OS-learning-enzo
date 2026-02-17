# 第六章编程练习 - 工作记录

## 一、硬链接（linkat / unlinkat / fstat）

### 1. tg-easy-fs 本地化与修改

- **克隆**：在 `tg-ch6` 下执行 `cargo clone tg-easy-fs`，并将 `Cargo.toml` / `Cargo.toml.orig` 中依赖改为 `tg-easy-fs = { path = "./tg-easy-fs" }`（含 build-dependencies）。
- **layout.rs**
  - 在 `DiskInode` 中新增字段 `nlink: u32`。
  - `initialize()` 中设置 `self.nlink = 1`，并增加 `nlink()` 方法。
- **efs.rs**
  - 新增 `dealloc_inode(&mut self, inode_id: u32)`，用于释放 inode 位图。
- **vfs.rs**
  - **Inode** 增加 `inode_id: u32`，`Inode::new` 及所有构造处（含 `efs::root_inode`）传入 `inode_id`；新增 `inode_id()`、`nlink()`、`is_dir()`（供 fstat 使用）。
  - **find_inode_id**：遍历目录项时跳过 `inode_number() == 0` 的条目；将 `assert!(disk_inode.is_dir())` 改为 `if !disk_inode.is_dir() { return None }`，避免旧镜像布局不一致时 panic。
  - **readdir**：跳过 `inode_number() == 0` 的目录项。
  - 新增 **get_inode_by_id(inode_id)**、**add_dirent(name, inode_id)**、**increment_nlink()**、**decrement_nlink_and_maybe_free(inode_id)**、**remove_dirent(name)**、**link(src, dst)**、**unlink(name)**，实现硬链接的创建与删除（含 nlink 增减与 nlink=0 时回收 inode 与数据块）。

### 2. 内核侧

- **fs.rs**：实现 `FSManager::link` 与 `FSManager::unlink`，分别委托 `self.root.link(src, dst)` 和 `self.root.unlink(path)`，返回值 0 / -1。
- **main.rs（impl IO）**
  - **linkat**：从用户空间读取 oldpath、newpath（与 open 相同的逐字节读法）；若 oldpath == newpath 返回 -1，否则调用 `FS.link(old, new)`。
  - **unlinkat**：从用户空间读取 path，调用 `FS.unlink(path)`。
  - **fstat**：根据 fd 取当前进程的 `fd_table[fd]` 及其中 inode；用 `Stat::new()` 填 `dev=0`、`ino=inode.inode_id()`、`mode=StatMode::DIR/FILE`、`nlink=inode.nlink()`；将结果写回用户态 `st`（地址翻译后写入，参考 clock_gettime 的 TimeSpec 写法）。

### 3. 构建与镜像

- **build.rs**：增加 `cargo:rerun-if-changed=tg-easy-fs`，使修改 tg-easy-fs 后重新打包 `fs.img`，保证磁盘布局与当前 `DiskInode` 一致。

---

## 二、测试与注意点

- 若曾用旧版 tg-easy-fs 打过 `fs.img`，建议先 `cargo clean && cargo build` 再测，避免根 inode 布局不一致。
- 练习测例：`cargo run --features exercise` 后在内核 shell 输入 `ch6_usertest`；或执行 `./test.sh exercise`。
