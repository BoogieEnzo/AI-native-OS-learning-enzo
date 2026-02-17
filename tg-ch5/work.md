# mmap / munmap 实现记录

## 写了什么

- **位置**：`src/main.rs` 里 `impl Memory for SyscallContext` 的 `mmap`、`munmap` 两个函数；impls 里增加了对 `parse_flags`、`PROTAL_TRANSIT` 的引用。
- **依赖**：当前进程用 `PROCESSOR.get_mut().current()` 得到 `&mut Process`，在其 `address_space` 上做 `map` / `unmap`（和 sbrk、exec 一样，只动当前进程的地址空间）。

## 思考逻辑

### 1. 当前进程的可变引用

- 需要改当前进程的 `address_space`，所以要 `&mut Process`。
- ch5 里 `PManager::current()` 已经返回 `Option<&mut P>`，直接用即可；不需要再通过 pid 调 `get_mut(pid)`（且 PManager 对外没有暴露 get_mut，那是 Manage 在内部用的）。

### 2. mmap 的约束与错误

- **addr 对齐**：必须 `addr % PAGE_SIZE == 0`，否则 -1。
- **prot**：只允许 bit0/1/2（R/W/X），且不能全 0（至少一个权限），其它位必须 0 → `prot & !0b111 != 0` 或 `prot & 0b111 == 0` 则 -1。
- **len**：题设“len 可为 0，按页向上取整”；和 ch4 一致，len==0 当 1 页处理，否则 `(len + PAGE_MASK) & !PAGE_MASK`。
- **已映射**：不能覆盖已有映射。用 `translate(页起始 VA, VALID)` 逐页查；任意一页 `Some` 就 -1。不用 `areas` 是避免依赖 tg-kernel-vm 内部结构，用 translate 更稳。
- **传送门**：`PROTAL_TRANSIT` 所在页不能给用户 mmap，否则会破坏 Trap 返回路径。判断 `vpn_range` 与 `PROTAL_TRANSIT` 是否相交，相交则 -1。

### 3. 权限到 VmFlags

- 题设：bit0=R, bit1=W, bit2=X。ELF/process 里用的是 "U_XWR_V" 这种字符串（U、X/W/R、V）。用 5 字节数组 `[b'U',b'_',b'_',b'_',b'V']` 按 prot 填 X/W/R，再 `parse_flags(from_utf8_unchecked(&buf))` 得到 `VmFlags`，避免在 no_std 里拼 String。

### 4. munmap 的约束与错误

- **addr 对齐**：同 mmap。
- **len 取整**：同 mmap。
- **“存在未被映射的虚存”**：区间内每一页都必须已经映射。逐页 `translate(..., VALID)`，任意一页 `None` 就 -1，再调用 `address_space.unmap(vpn_range)`。

### 5. VPN 区间

- `vpn_range = vaddr_start.floor()..vaddr_end.ceil()`，和 process 里 sbrk、from_elf 一致。
- 遍历页用 `vpn_range.start.val()..vpn_range.end.val()` 得到页号，再 `page_va = i << PAGE_BITS` 得到每页起始虚地址给 translate 用。

### 6. 小结

- 先严格校验参数和区间（对齐、prot、已映射/未映射），再调 `address_space.map(..., &[], 0, flags)` / `unmap(...)`。
- 不维护 trace，不碰 fd/offset；匿名映射用空数据 `&[]`，offset 0。

---

## spawn（5.2）

- **位置**：`main.rs` 里 `impl Process for SyscallContext::spawn`。
- **逻辑**：用 `current().address_space.translate(path, READABLE)` 取程序名字符串 → `APPS.get(name)` 取 ELF → `ProcStruct::from_elf(elf)` 建进程 → `(*processor).add(pid, child, parent_pid)` 加入调度，父进程为 current。失败（名字无效、ELF 错、from_elf 失败）返回 -1。
- **作用**：ch5_usertest 用 spawn 跑 ch4_mmap 等子测例，不实现 spawn 则练习测例不会真正执行。

---

## stride 调度 + set_priority（5.3）

### stride 是什么

- 每个进程有两个数：**stride**（当前已跑的“步数”）和 **pass**（每次被调度后要加的步数，pass = BigStride / priority）。
- **调度规则**：每次要选进程时，在就绪队列里选 **stride 最小** 的进程运行；该进程跑完一段时间回到就绪队列前，把它的 stride 加上自己的 pass。
- **效果**：priority 越大，pass 越小，stride 涨得越慢，就越容易被选到，所以 **得到 CPU 的时间与 priority 成正比**。BigStride 是常数（这里用 65536），只影响步长单位。

### 代码改动

- **process.rs**：`Process` 增加 `stride: usize`、`priority: usize`。`from_elf` 里新进程 stride=0、priority=16；`fork` 里继承父进程的 stride、priority；`exec` 不改这两项（保留当前进程的优先级）。
- **processor.rs**：`fetch()` 不再 `pop_front()`。遍历 `ready_queue` 找 stride 最小的 id，从队列删掉该 id，对该进程做 `stride += BIG_STRIDE / priority.max(1)`，返回该 id。`add()` 不变。
- **main.rs**：`set_priority(prio)`：prio 小于 2 则返回 -1，否则设 `current.priority = prio` 并返回 prio。

### test.sh

- 练习测试需要 shell 里跑 `ch5_usertest`，脚本里用 `printf 'ch5_usertest\n' | timeout 120 cargo run --features exercise ...` 把命令喂给 QEMU 标准输入，否则测例不会跑。
