# 至少 5 个基础实验：路线与验收（基于 rCore 组件化实验）

本文件给出一个**最小可验收**的“5 个基础实验”集合：`ch1` 到 `ch5`。

实际节奏建议：**每周完成 1 个 chapter**，5 周完成 5 个（ch1~ch5）。

每个实验都要求你留下两类证据：

- **可复现命令**：别人照抄能跑出同样结果
- **验收证据**：输出片段/截图，或 `tg-checker` 通过

> 假设你的本地目录结构类似：
>
> ```text
> ~/AI-native-OS-learning-enzo/
>   ai4ose-lab1-2026s/
>   rCore-Tutorial-in-single-workspace/
> ```
>
> 且你当前所在目录为 `ai4ose-lab1-2026s/`，下面所有 `cd ../rCore-Tutorial-in-single-workspace/...` 的命令都可以直接使用。

## 前置：安装 `tg-checker`（一次即可）

在 `rCore-Tutorial-in-single-workspace` 目录中：

```bash
cargo install --path tg-checker
```

验收：

```bash
tg-checker --list
```

## 实验 1：`ch1` 最小内核启动与 SBI 输出

- **目标**：理解“入口 `_start`、设置栈、跳转到 Rust、通过 SBI 打印、关机”的最小闭环
- **代码位置**：`rCore-Tutorial-in-single-workspace/ch1/`
- **运行命令**：

```bash
cd ../rCore-Tutorial-in-single-workspace/ch1
cargo run
```

- **验收证据**：输出包含 `Hello, world!`，并正常退出（QEMU 关机）

## 实验 2：`ch2` 基础功能跑通 + `tg-checker` 验收

```bash
cd ../rCore-Tutorial-in-single-workspace/ch2
cargo run 2>&1 | tg-checker --ch 2
```

- **验收证据**：`tg-checker` 返回通过（退出码为 0）

## 实验 3：`ch3` 基础功能跑通 + `tg-checker` 验收

```bash
cd ../rCore-Tutorial-in-single-workspace/ch3
cargo run 2>&1 | tg-checker --ch 3
```

## 实验 4：`ch4` 基础功能跑通 + `tg-checker` 验收

```bash
cd ../rCore-Tutorial-in-single-workspace/ch4
cargo run 2>&1 | tg-checker --ch 4
```

## 实验 5：`ch5` 基础功能跑通 + `tg-checker` 验收

```bash
cd ../rCore-Tutorial-in-single-workspace/ch5
cargo run 2>&1 | tg-checker --ch 5
```

## 进阶（可选）：Exercise 模式

如果你想把“做题/补全”也纳入验收，可以对 `ch3/ch4/ch5/...` 使用 `--features exercise`，并用 `--exercise` 校验：

```bash
cd ../rCore-Tutorial-in-single-workspace/ch3
cargo run --features exercise 2>&1 | tg-checker --ch 3 --exercise
```

## 把证据放到哪里

放到**当周对应的 lab 仓库**里（例如 lab2、lab3…）。建议每个 chapter 留一页记录，包含：命令、输出片段/截图、你理解到的 3 个关键点、1 个遗留问题。

