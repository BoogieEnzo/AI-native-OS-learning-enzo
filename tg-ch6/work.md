# 第六章编程练习 - 工作记录

## 一、硬链接（linkat / unlinkat / fstat）

### 1. tg-easy-fs 本地化与修改

- **克隆**：在 `tg-ch6` 下执行 `cargo clone tg-easy-fs`，并将 `Cargo.toml` / `Cargo.toml.orig` 中依赖改为 `tg-easy-fs = { path = "./tg-easy-fs" }`（含 build-dependencies）。
- **layout.rs**
  - 在 `DiskInode` 中新增字段 `nlink: u32`。
  - `initialize()` 中设置 `self.nlink = 1`，并增加 `nlink()` 方法。
  
  ```rust
  // tg-easy-fs/src/layout.rs
  pub struct DiskInode {
      pub size: u32,
      pub direct: [u32; INODE_DIRECT_COUNT],
      pub indirect1: u32,
      pub indirect2: u32,
      type_: DiskInodeType,
      /// 硬链接数量，新建时为 1
      pub nlink: u32,  // 新增字段
  }
  
  impl DiskInode {
      pub fn initialize(&mut self, type_: DiskInodeType) {
          self.size = 0;
          self.direct.iter_mut().for_each(|v| *v = 0);
          self.indirect1 = 0;
          self.indirect2 = 0;
          self.type_ = type_;
          self.nlink = 1;  // 初始化时设置为 1
      }
      
      /// 获取硬链接数
      pub fn nlink(&self) -> u32 {
          self.nlink
      }
  }
  ```

- **efs.rs**
  - 新增 `dealloc_inode(&mut self, inode_id: u32)`，用于释放 inode 位图。
  - 修改 `root_inode` 方法，传入 `inode_id = 0`（根目录固定为 inode 0）。
  
  ```rust
  // tg-easy-fs/src/efs.rs
  impl EasyFileSystem {
      /// Get the root inode of the filesystem
      pub fn root_inode(efs: &Arc<Mutex<Self>>) -> Inode {
          let block_device = Arc::clone(&efs.lock().block_device);
          // acquire efs lock temporarily
          let (block_id, block_offset) = efs.lock().get_disk_inode_pos(0);
          // release efs lock
          Inode::new(block_id, block_offset, 0, Arc::clone(efs), block_device)  // 传入 inode_id = 0
      }
      
      /// Deallocate an inode (release inode bitmap slot).
      pub fn dealloc_inode(&mut self, inode_id: u32) {
          self.inode_bitmap
              .dealloc(&self.block_device, inode_id as usize);
      }
  }
  ```

- **vfs.rs**
  - **Inode** 增加 `inode_id: u32`，`Inode::new` 及所有构造处（含 `efs::root_inode`）传入 `inode_id`；新增 `inode_id()`、`nlink()`、`is_dir()`（供 fstat 使用）。
  
  ```rust
  // tg-easy-fs/src/vfs.rs
  pub struct Inode {
      block_id: usize,
      block_offset: usize,
      inode_id: u32,  // 新增字段
      fs: Arc<Mutex<EasyFileSystem>>,
      block_device: Arc<dyn BlockDevice>,
  }
  
  impl Inode {
      pub fn new(
          block_id: u32,
          block_offset: usize,
          inode_id: u32,  // 新增参数
          fs: Arc<Mutex<EasyFileSystem>>,
          block_device: Arc<dyn BlockDevice>,
      ) -> Self {
          Self {
              block_id: block_id as usize,
              block_offset,
              inode_id,  // 存储 inode_id
              fs,
              block_device,
          }
      }
      
      /// Inode 编号（用于 fstat 等）
      pub fn inode_id(&self) -> u32 {
          self.inode_id
      }
      
      /// 硬链接数
      pub fn nlink(&self) -> u32 {
          self.read_disk_inode(|d| d.nlink())
      }
      
      /// 是否为目录
      pub fn is_dir(&self) -> bool {
          self.read_disk_inode(|d| d.is_dir())
      }
  }
  ```
  
  - **find_inode_id**：遍历目录项时跳过 `inode_number() == 0` 的条目；将 `assert!(disk_inode.is_dir())` 改为 `if !disk_inode.is_dir() { return None }`，避免旧镜像布局不一致时 panic。
  
  ```rust
  fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
      if !disk_inode.is_dir() {  // 改为 if 判断，避免 panic
          return None;
      }
      let file_count = (disk_inode.size as usize) / DIRENT_SZ;
      let mut dirent = DirEntry::empty();
      for i in 0..file_count {
          assert_eq!(
              disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
              DIRENT_SZ,
          );
          if dirent.inode_number() != 0 && dirent.name() == name {  // 跳过 inode_number == 0
              return Some(dirent.inode_number());
          }
      }
      None
  }
  ```
  
  - **readdir**：跳过 `inode_number() == 0` 的目录项。
  
  ```rust
  pub fn readdir(&self) -> Vec<String> {
      let _fs = self.fs.lock();
      self.read_disk_inode(|disk_inode| {
          let file_count = (disk_inode.size as usize) / DIRENT_SZ;
          let mut v: Vec<String> = Vec::new();
          for i in 0..file_count {
              let mut dirent = DirEntry::empty();
              assert_eq!(
                  disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                  DIRENT_SZ,
              );
              if dirent.inode_number() != 0 {  // 跳过 inode_number == 0
                  v.push(String::from(dirent.name()));
              }
          }
          v
      })
  }
  ```
  
  - 新增 **get_inode_by_id(inode_id)**、**add_dirent(name, inode_id)**、**increment_nlink()**、**decrement_nlink_and_maybe_free(inode_id)**、**remove_dirent(name)**、**link(src, dst)**、**unlink(name)**，实现硬链接的创建与删除（含 nlink 增减与 nlink=0 时回收 inode 与数据块）。
  
  ```rust
  /// 根据 inode_id 获取 Inode（用于硬链接操作）
  pub fn get_inode_by_id(&self, inode_id: u32) -> Arc<Inode> {
      let fs = self.fs.lock();
      let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
      Arc::new(Self::new(
          block_id,
          block_offset,
          inode_id,
          self.fs.clone(),
          self.block_device.clone(),
      ))
  }
  
  /// 在目录中追加一个目录项（不分配新 inode，用于硬链接）
  pub fn add_dirent(&self, name: &str, inode_id: u32) {
      let mut fs = self.fs.lock();
      self.modify_disk_inode(|root_inode| {
          let file_count = (root_inode.size as usize) / DIRENT_SZ;
          let new_size = (file_count + 1) * DIRENT_SZ;
          self.increase_size(new_size as u32, root_inode, &mut fs);
          let dirent = DirEntry::new(name, inode_id);
          root_inode.write_at(
              file_count * DIRENT_SZ,
              dirent.as_bytes(),
              &self.block_device,
          );
      });
      block_cache_sync_all();
  }
  
  /// 增加硬链接计数
  pub fn increment_nlink(&self) {
      self.modify_disk_inode(|d| d.nlink += 1);
  }
  
  /// 减少硬链接计数，若为 0 则释放 inode 及数据块
  pub fn decrement_nlink_and_maybe_free(&self, inode_id: u32) {
      let fs = self.fs.lock();
      let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
      let nlink = get_block_cache(block_id as usize, Arc::clone(&self.block_device))
          .lock()
          .modify(block_offset, |d: &mut DiskInode| {
              d.nlink = d.nlink.saturating_sub(1);
              d.nlink
          });
      drop(fs);
      if nlink == 0 {
          let target = self.get_inode_by_id(inode_id);
          target.clear();
          self.fs.lock().dealloc_inode(inode_id);
      }
      block_cache_sync_all();
  }
  
  /// 从目录中移除指定名称的目录项（置空），返回被删条目的 inode_id
  pub fn remove_dirent(&self, name: &str) -> Option<u32> {
      let (idx, inode_id) = self.read_disk_inode(|disk_inode| {
          let file_count = (disk_inode.size as usize) / DIRENT_SZ;
          for i in 0..file_count {
              let mut dirent = DirEntry::empty();
              assert_eq!(
                  disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device),
                  DIRENT_SZ,
              );
              if dirent.inode_number() != 0 && dirent.name() == name {
                  return Some((i, dirent.inode_number()));
              }
          }
          None
      })?;
      self.modify_disk_inode(|root| {
          let empty = DirEntry::empty();
          root.write_at(DIRENT_SZ * idx, empty.as_bytes(), &self.block_device);
      });
      Some(inode_id)
  }
  
  /// 创建硬链接：dst 作为 src 的另一个名字指向同一 inode（仅支持普通文件）
  pub fn link(&self, src: &str, dst: &str) -> Option<()> {
      if src == dst {
          return None;
      }
      let inode_id = self.read_disk_inode(|disk_inode| self.find_inode_id(src, disk_inode))?;
      let target = self.get_inode_by_id(inode_id);
      let is_file = target.read_disk_inode(|d| d.is_file());
      if !is_file {
          return None;
      }
      self.add_dirent(dst, inode_id);
      target.increment_nlink();
      Some(())
  }
  
  /// 删除目录项并减少 nlink，若 nlink 为 0 则释放 inode
  pub fn unlink(&self, name: &str) -> Option<()> {
      let inode_id = self.remove_dirent(name)?;
      let target = self.get_inode_by_id(inode_id);
      target.decrement_nlink_and_maybe_free(inode_id);
      Some(())
  }
  ```

### 2. 内核侧

- **fs.rs**：实现 `FSManager::link` 与 `FSManager::unlink`，分别委托 `self.root.link(src, dst)` 和 `self.root.unlink(path)`，返回值 0 / -1。
  
  ```rust
  // src/fs.rs
  impl FSManager for FileSystem {
      /// 创建硬链接
      fn link(&self, src: &str, dst: &str) -> isize {
          if self.root.link(src, dst).is_some() {
              0
          } else {
              -1
          }
      }
      
      /// 删除硬链接
      fn unlink(&self, path: &str) -> isize {
          if self.root.unlink(path).is_some() {
              0
          } else {
              -1
          }
      }
  }
  ```

- **main.rs（impl IO）**
  - **linkat**：从用户空间读取 oldpath、newpath（与 open 相同的逐字节读法）；若 oldpath == newpath 返回 -1，否则调用 `FS.link(old, new)`。
  
  ```rust
  // src/main.rs (impl IO for SyscallContext)
  fn linkat(
      &self,
      _caller: Caller,
      _olddirfd: i32,
      oldpath: usize,
      _newdirfd: i32,
      newpath: usize,
      _flags: u32,
  ) -> isize {
      let current = PROCESSOR.get_mut().current().unwrap();
      let read_path = |path_ptr: usize| -> Option<String> {
          let ptr = current.address_space.translate(VAddr::new(path_ptr), READABLE)?;
          let mut s = String::new();
          let mut raw_ptr: *const u8 = ptr.as_ptr();
          loop {
              let ch = unsafe { *raw_ptr };
              if ch == 0 {
                  break;
              }
              s.push(ch as char);
              raw_ptr = unsafe { raw_ptr.add(1) };
          }
          Some(s)
      };
      let Some(old) = read_path(oldpath) else {
          return -1;
      };
      let Some(new) = read_path(newpath) else {
          return -1;
      };
      if old == new {
          return -1;
      }
      FS.link(old.as_str(), new.as_str())
  }
  ```
  
  - **unlinkat**：从用户空间读取 path，调用 `FS.unlink(path)`。
  
  ```rust
  fn unlinkat(&self, _caller: Caller, _dirfd: i32, path: usize, _flags: u32) -> isize {
      let current = PROCESSOR.get_mut().current().unwrap();
      let Some(ptr) = current.address_space.translate(VAddr::new(path), READABLE) else {
          return -1;
      };
      let mut s = String::new();
      let mut raw_ptr: *const u8 = ptr.as_ptr();
      loop {
          let ch = unsafe { *raw_ptr };
          if ch == 0 {
              break;
          }
          s.push(ch as char);
          raw_ptr = unsafe { raw_ptr.add(1) };
      }
      FS.unlink(s.as_str())
  }
  ```
  
  - **fstat**：根据 fd 取当前进程的 `fd_table[fd]` 及其中 inode；用 `Stat::new()` 填 `dev=0`、`ino=inode.inode_id()`、`mode=StatMode::DIR/FILE`、`nlink=inode.nlink()`；将结果写回用户态 `st`（地址翻译后写入，参考 clock_gettime 的 TimeSpec 写法）。
  
  ```rust
  fn fstat(&self, _caller: Caller, fd: usize, st: usize) -> isize {
      let current = PROCESSOR.get_mut().current().unwrap();
      if fd >= current.fd_table.len() {
          return -1;
      }
      let file_guard = match &current.fd_table[fd] {
          Some(f) => f,
          None => return -1,
      };
      let file = file_guard.lock();
      let inode = match &file.inode {
          Some(inode) => inode,
          None => return -1,
      };
      let mode = if inode.is_dir() {
          tg_syscall::StatMode::DIR
      } else {
          tg_syscall::StatMode::FILE
      };
      let mut stat = tg_syscall::Stat::new();
      stat.dev = 0;
      stat.ino = inode.inode_id() as u64;
      stat.mode = mode;
      stat.nlink = inode.nlink();
      if let Some(mut ptr) = current
          .address_space
          .translate::<tg_syscall::Stat>(VAddr::new(st), WRITEABLE)
      {
          unsafe {
              *ptr.as_mut() = stat;
          }
          0
      } else {
          -1
      }
  }
  ```

### 3. 构建与镜像

- **build.rs**：增加 `cargo:rerun-if-changed=tg-easy-fs`，使修改 tg-easy-fs 后重新打包 `fs.img`，保证磁盘布局与当前 `DiskInode` 一致。

---

## 二、测试与注意点

- 若曾用旧版 tg-easy-fs 打过 `fs.img`，建议先 `cargo clean && cargo build` 再测，避免根 inode 布局不一致。
- 练习测例：`cargo run --features exercise` 后在内核 shell 输入 `ch6_usertest`；或执行 `./test.sh exercise`。
