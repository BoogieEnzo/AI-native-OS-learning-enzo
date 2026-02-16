# ch4 代码改动与 Bug 分析总结

## 零、代码实现（从零开始）

本章需要实现三个系统调用：`trace`、`mmap`、`munmap`。原始代码库中这些系统调用只有占位实现（返回 -1 或打印 "not implemented"），需要从零开始实现完整功能。

### 0.1 添加系统调用统计支持（process.rs）

**原代码：** `Process` 结构体只有 4 个字段（`context`、`address_space`、`heap_bottom`、`program_brk`），没有系统调用计数功能。

**实现：**

1. **定义常量**（process.rs 第 36-37 行）：
```rust
/// 系统调用计数数组大小
const SYSCALL_COUNT_CAP: usize = 512;
```

2. **扩展 Process 结构体**（process.rs 第 47-58 行）：
```rust
pub struct Process {
    pub context: ForeignContext,
    pub address_space: AddressSpace<Sv39, Sv39Manager>,
    pub heap_bottom: usize,
    pub program_brk: usize,
    /// 系统调用计数数组（新增）
    pub syscall_count: [u32; SYSCALL_COUNT_CAP],
}
```

3. **初始化 syscall_count**（process.rs 第 159 行）：
```rust
Some(Self {
    context: ForeignContext { context, satp },
    address_space,
    heap_bottom,
    program_brk: heap_bottom,
    syscall_count: [0; SYSCALL_COUNT_CAP],  // 初始化为全 0
})
```

**原因：** `trace` 系统调用的 `trace_request=2` 需要查询系统调用次数，需要在进程级别维护计数数组。

---

### 0.2 在调度器中添加系统调用统计（main.rs）

**原代码：** `schedule()` 函数中系统调用处理直接调用 `tg_syscall::handle()`，没有统计逻辑。

**实现：**（main.rs 第 250-259 行）

在系统调用分发**之前**统计调用次数：

```rust
let ctx = &mut ctx.context;
let id: Id = ctx.a(7).into();
let args = [ctx.a(0), ctx.a(1), ctx.a(2), ctx.a(3), ctx.a(4), ctx.a(5)];

// 统计系统调用次数（在分发前自增）
let process = unsafe { &mut PROCESSES.get_mut()[0] };
let id_raw = id.0;
if id_raw < 512 {
    process.syscall_count[id_raw] = process.syscall_count[id_raw].saturating_add(1);
}

match tg_syscall::handle(Caller { entity: 0, flow: 0 }, id, args) {
    // ...
}
```

**关键点：**
- 统计在 `handle()` **之前**进行，确保 `trace_request=2` 查询时能包含本次调用
- 使用 `saturating_add(1)` 防止溢出
- 通过 `PROCESSES.get_mut()[0]` 访问当前进程（`caller.entity = 0`）

---

### 0.3 实现 trace 系统调用（main.rs）

**原代码：**（main.rs 第 564-576 行）
```rust
impl Trace for SyscallContext {
    #[inline]
    fn trace(&self, _caller: Caller, _trace_request: usize, _id: usize, _data: usize) -> isize {
        tg_console::log::info!("trace: not implemented");
        -1
    }
}
```

**实现：**（main.rs 第 574-622 行）

根据 `trace_request` 参数实现三个功能：

```rust
impl Trace for SyscallContext {
    fn trace(&self, caller: Caller, trace_request: usize, id: usize, data: usize) -> isize {
        let process = match unsafe { PROCESSES.get_mut() }.get_mut(caller.entity) {
            Some(p) => p,
            None => return -1,
        };
        
        match trace_request {
            // 0: 读取用户内存（id 视为 *const u8，返回 1 字节）
            0 => {
                const READABLE: VmFlags<Sv39> = build_flags("URV");
                if let Some(ptr) = process.address_space
                    .translate::<u8>(VAddr::new(id), READABLE)
                {
                    unsafe { ptr.as_ptr().read() as isize }
                } else {
                    -1  // 地址不可读或不存在
                }
            }
            // 1: 写入用户内存（id 视为 *mut u8，写入 data 低 8 位）
            1 => {
                const WRITABLE: VmFlags<Sv39> = build_flags("UWV");
                if let Some(ptr) = process.address_space
                    .translate::<u8>(VAddr::new(id), WRITABLE)
                {
                    unsafe { (ptr.as_ptr() as *mut u8).write(data as u8) };
                    0
                } else {
                    -1  // 地址不可写或不存在
                }
            }
            // 2: 查询系统调用 id 的调用次数
            2 => {
                if id < process.syscall_count.len() {
                    process.syscall_count[id] as isize
                } else {
                    0
                }
            }
            _ => -1,
        }
    }
}
```

**关键点：**
- **trace_request=0/1**：使用 `address_space.translate()` 进行地址翻译和权限检查
  - 权限标志：`"URV"`（用户可读有效）、`"UWV"`（用户可写有效）
  - 翻译失败返回 -1（而非 panic）
- **trace_request=2**：直接从 `process.syscall_count` 数组读取
- 参考 `clock_gettime` 的实现模式（使用 `translate()` + 权限检查）

---

### 0.4 实现 mmap 系统调用（main.rs）

**原代码：**（main.rs 第 582-597 行）
```rust
impl Memory for SyscallContext {
    fn mmap(&self, _caller: Caller, addr: usize, len: usize, prot: i32, ...) -> isize {
        tg_console::log::info!("mmap: addr = {addr:#x}, len = {len}, prot = {prot}, not implemented");
        -1
    }
}
```

**实现：**（main.rs 第 628-697 行）

完整实现参数检查、地址映射和权限设置：

```rust
fn mmap(&self, caller: Caller, addr: usize, len: usize, prot: i32, ...) -> isize {
    let process = match unsafe { PROCESSES.get_mut() }.get_mut(caller.entity) {
        Some(p) => p,
        None => return -1,
    };
    
    const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
    const PAGE_MASK: usize = PAGE_SIZE - 1;
    
    // 1. 检查 addr 是否按页对齐
    if addr & PAGE_MASK != 0 {
        return -1;
    }
    
    // 2. 检查 prot 参数有效性
    if prot & !0b111 != 0 {  // 其他位必须为 0
        return -1;
    }
    if prot & 0b111 == 0 {   // 至少需要 R/W/X 之一
        return -1;
    }
    
    // 3. len 处理：0 时按页向上取整为 1 页
    let len = if len == 0 { PAGE_SIZE } else { len };
    let len_aligned = (len + PAGE_MASK) & !PAGE_MASK;
    
    // 4. 计算 VPN 范围
    let vaddr_start = VAddr::<Sv39>::new(addr);
    let vaddr_end = VAddr::<Sv39>::new(addr + len_aligned);
    let vpn_range = vaddr_start.floor()..vaddr_end.ceil();
    
    // 5. 检查地址范围是否已映射（遍历 areas）
    for area in &process.address_space.areas {
        if area.start < vpn_range.end && area.end > vpn_range.start {
            return -1;  // 有重叠
        }
    }
    
    // 6. 构建页表项权限标志（从 prot 转换为 VmFlags）
    let mut flags_str = String::from("U_V");  // 用户态 + 有效
    if prot & 0b001 != 0 { flags_str.insert(2, 'R'); }  // 可读
    if prot & 0b010 != 0 { flags_str.insert(2, 'W'); }  // 可写
    if prot & 0b100 != 0 { flags_str.insert(2, 'X'); }  // 可执行
    
    let flags = match parse_flags(&flags_str) {
        Ok(f) => f,
        Err(_) => return -1,
    };
    
    // 7. 分配物理页并映射
    process.address_space.map(vpn_range, &[], 0, flags);
    0
}
```

**关键点：**
- **参数检查**：地址对齐、prot 有效性（`prot & !0x7 == 0` 且 `prot & 0x7 != 0`）
- **地址重叠检查**：遍历 `address_space.areas`，检查是否有重叠区域
- **权限转换**：将 `prot`（bit 0=R, 1=W, 2=X）转换为 `VmFlags` 字符串（如 `"U_WRV"`）
- **映射操作**：使用 `address_space.map()` 分配物理页并建立映射
- **必须添加 `U` 标志**：用户态可访问

---

### 0.5 实现 munmap 系统调用（main.rs）

**原代码：**（main.rs 第 599-602 行）
```rust
fn munmap(&self, _caller: Caller, addr: usize, len: usize) -> isize {
    tg_console::log::info!("munmap: addr = {addr:#x}, len = {len}, not implemented");
    -1
}
```

**实现：**（main.rs 第 699-743 行）

实现地址范围验证和取消映射：

```rust
fn munmap(&self, caller: Caller, addr: usize, len: usize) -> isize {
    let process = match unsafe { PROCESSES.get_mut() }.get_mut(caller.entity) {
        Some(p) => p,
        None => return -1,
    };
    
    const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
    const PAGE_MASK: usize = PAGE_SIZE - 1;
    
    // 1. 检查 addr 是否按页对齐
    if addr & PAGE_MASK != 0 {
        return -1;
    }
    
    // 2. len 处理
    let len = if len == 0 { PAGE_SIZE } else { len };
    let len_aligned = (len + PAGE_MASK) & !PAGE_MASK;
    
    // 3. 计算 VPN 范围
    let vaddr_start = VAddr::<Sv39>::new(addr);
    let vaddr_end = VAddr::<Sv39>::new(addr + len_aligned);
    let vpn_range = vaddr_start.floor()..vaddr_end.ceil();
    
    // 4. 检查地址范围是否完全在已映射区域内
    // 需要确保 [vpn_range.start, vpn_range.end) 范围内的每一页都被映射
    let mut vpn = vpn_range.start;
    while vpn < vpn_range.end {
        let mut found = false;
        for area in &process.address_space.areas {
            if area.start <= vpn && area.end > vpn {
                found = true;
                break;
            }
        }
        if !found {
            return -1;  // 存在未映射的页
        }
        vpn = vpn + 1;
    }
    
    // 5. 取消映射
    process.address_space.unmap(vpn_range);
    0
}
```

**关键点：**
- **地址对齐检查**：与 `mmap` 相同
- **完整性检查**：遍历范围内的每一页，确保都在某个 `area` 内
  - 不能只检查范围边界，必须逐页验证（避免部分映射的情况）
- **取消映射**：使用 `address_space.unmap()` 清除页表项

---

### 0.6 导入依赖（main.rs）

**新增导入：**（main.rs 第 369-370 行）
```rust
use crate::{build_flags, parse_flags, Sv39, PROCESSES};
use alloc::{alloc::alloc_zeroed, string::String};  // String 用于构建 flags_str
```

---

## 一、改动代码

### 1. 调度栈扩容（main.rs 第 192–196 行）

**原代码：**
```rust
const PAGE: Layout = unsafe { Layout::from_size_align_unchecked(2 << Sv39::PAGE_BITS, 1 << Sv39::PAGE_BITS) };
let pages = 2;
```

**修改后：**
```rust
const PAGE: Layout = unsafe { Layout::from_size_align_unchecked(6 << Sv39::PAGE_BITS, 1 << Sv39::PAGE_BITS) };
let pages = 6;
```

**原因：** 调度栈 2 页（8 KiB）不够，`schedule()`、`ctx.execute(portal)` 及 trap 处理深度会触发栈溢出。

---

### 2. mmap prot=0 检查（main.rs 第 656–659 行）

**新增：**
```rust
// prot 至少需要 R/W/X 之一，全 0 无意义
if prot & 0b111 == 0 {
    return -1;
}
```

**原因：** 规范要求 `prot & 0x7 = 0` 时返回 -1，ch4_mmap3 第 21 行会验证 `mmap(..., 0)` 返回 -1。

---

## 二、Bug 与排查

### Bug 1：调度栈溢出导致 StorePageFault

**现象：**
```
[ERROR] stval = 0x3fffffdff8
[ERROR] panicked at src/main.rs:209:5: trap from scheduling thread: Exception(StorePageFault)
[FAIL] not found <Test write A OK!>
[FAIL] not found <Test write B OK!>
...
```

**分析：**

| 项目 | 值 |
|------|-----|
| stval | 0x3fffffdff8 |
| VPN | 0x3fffffdff8 >> 12 = 0x3FFFFFD (67108861) |
| 调度栈 VPN | (1<<26)-2..1<<26 = 67108862..67108864 |
| 栈基址 | 0x3FFFFE000 (67108862 页) |

stval 位于 VPN 67108861，而调度栈从 VPN 67108862 开始，说明访问发生在栈下方一页，属于**栈溢出**。

**根因：** 2 页调度栈过小，`schedule()` 和 trap 处理等调用链导致 SP 越过栈底。

---

### Bug 2：mmap prot=0 未拒绝

**现象：**
```
[ERROR] Panicked at src/bin/ch4_mmap3.rs:19: assertion `left == right` failed
  left: 0
 right: -1
[FAIL] not found <Test 04_4 test OK!>
```

**分析：**

- 失败断言：`assert_eq!(mmap(start + len, len, 0), -1);`（ch4_mmap3 第 21 行）
- 语义：`prot = 0` 表示无任何 R/W/X 权限，规范要求返回 -1
- 实际：mmap 返回 0（成功），与预期不符

**根因：** 未实现 `prot & 0x7 == 0` 的检查，导致非法 prot 被当作合法参数处理。

---

### 三、关于 StorePageFault / LoadPageFault @ 0x10000000

**现象：**
```
[ERROR] unsupported trap: Exception(StorePageFault), stval = 0x10000000
[ERROR] unsupported trap: Exception(LoadPageFault), stval = 0x10000000
```

**说明：** 这是预期行为。ch4_mmap1 对只读页写、ch4_mmap2 对非法权限页读，会触发异常，进程应被杀死。测试期望 `not found <Should cause error, Test 04_2 fail!>`，表示进程在打印错误前被正确终止。
