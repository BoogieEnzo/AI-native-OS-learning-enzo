# ch3 sys_trace 实验记录

## 任务目标

实现系统调用 `sys_trace`（ID 410），支持三种操作：
- `trace_request=0`：从用户地址 `id` 读 1 字节，返回值
- `trace_request=1`：向用户地址 `id` 写 1 字节（`data` 低 8 位），返回 0
- `trace_request=2`：查询当前任务对系统调用 `id` 的调用次数（本次计入），返回次数

---

## 实现内容

### 1. task.rs 修改

- 新增 `SYSCALL_COUNT_CAP = 512` 和 `syscall_count: [u32; SYSCALL_COUNT_CAP]`
- 在 `handle_syscall` 中：**分发前**对 `syscall_count[id]++`，并将 `syscall_count.as_ptr()` 通过 `Caller.entity` 传给 Trace 处理

### 2. main.rs 修改

- 实现 `Trace::trace`：根据 `trace_request` 分支，对 0/1 做地址读写，对 2 从 `caller.entity` 指向的数组取 `counts[id]`

---

## Bug：应用未加载 / 无用户程序输出

### 现象

运行 `cargo run --features exercise` 后：
- 只看到 `LOG TEST >> Hello, world!`，没有 `load app0 to...`
- 没有 `get_time OK!`、`Test trace OK!` 等用户程序输出
- 内核很快退出

### 已排除

- **apps 在 .data 段**，由 `app.asm` 编译进内核镜像，与栈无关
- 检查 `target/.../out/app.asm`，3 个应用（ch3_sleep, ch3_sleep1, ch3_trace）已正确生成并 `incbin` 嵌入
- 逻辑上若 apps 正常，`AppMeta::locate().iter()` 应返回 3 项，循环应打出 `load app0 to...`；未打出说明 iter 返回空

### 推理

1. **iter 为何空？** `AppIterator` 用 `meta.count` 控制循环，`count >= self.i` 时返回 None。若 `count` 被读成 0，则第一次 `next()` 就返回 None。
2. **count 为何变 0？** 未打栈、未 dump 内存，无直接证据。但唯一改动是 TCB 增加 `syscall_count[512]`（2KB/TCB）。
3. **tcbs 在哪？** `let mut tcbs = [TaskControlBlock::ZERO; APP_CAPACITY]` 是 `rust_main` 的局部变量 → **在栈上**。
4. **栈够吗？** 原 TCB 约 8.5KB，32 个 ≈ 272KB；加 syscall_count 后约 10.5KB/个，32 个 ≈ 336KB。内核栈 `STACK_SIZE = (32+2)*8192 = 272KB`。336KB > 272KB → 栈分配越界。
5. **越界后果？** 栈向下增长，越界会覆盖栈底以下区域（.bss 或 .data 尾部），可能破坏 `apps` 附近内存或其它关键数据，导致 `count` 被读成 0 或行为异常。
6. **反证**：增大 `STACK_SIZE` 加 64KB 后，`load app0` 和用户程序输出恢复 → 与「栈空间不足」的推断一致。

### 结论

栈溢出是**逻辑推断**（TCB 变大、tcbs 在栈、栈空间不足、改大栈后修复），未通过打栈或 dump 直接验证。

### 解决

增大内核栈：`STACK_SIZE = (APP_CAPACITY + 2) * 8192 + 64 * 1024`。

---

## 验证

```bash
cargo run --features exercise
./test.sh exercise
```

预期输出包含 `load app0 to 0x80400000` 及 `Test trace OK!` 等，7 个模式匹配全部通过。
