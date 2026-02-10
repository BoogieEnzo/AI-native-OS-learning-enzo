# ai4ose-lab1-2026s

[![Crates.io](https://img.shields.io/crates/v/ai4ose-lab1-2026s.svg)](https://crates.io/crates/ai4ose-lab1-2026s)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)

AI4OSE Lab1: 与 AI 合作进行操作系统内核学习的起点。

本仓库同时扮演两个角色：

- **可发布 crate**：`cargo install ai4ose-lab1-2026s` 后运行可打印实验说明（`src/content.txt`）。
- **教学实验环境（重点）**：把“怎么学/学什么/如何验收/如何记录 AI 协作与周报”固化在 `docs/` 与 `logs/` 中，帮助你持续迭代出适合自己的 OS 内核学习路径。

> 本仓库的教学环境以 “最新的 OS 课组件化实验代码” `rCore-Tutorial-in-single-workspace` 为**实验代码基座**。
> 你本地已经有该仓库（含 `ch1..ch8` 与 `tg-*` 组件 crate），因此这里不重复拷贝代码，而是给出**学习路线 + 验收方式 + 记录规范**。

## **快速浏览**

- **实验一说明（原文）**：`src/content.txt`
- **从“做实验”开始**：`docs/START.md`

## **教学实验环境入口**

- **总入口**：`docs/START.md`
- **至少 5 个基础实验的学习路线与验收**：`docs/5-basic-labs.md`

## **推荐的本地目录摆放**

你可以在本地创建一个“总的工作目录”，把 rCore 实验代码和本仓库都放在里面，例如你当前的布局：

```text
~/AI-native-OS-learning-enzo/                 # 你的本地工作总目录（可以本身也是一个 git 仓库）
  rCore-Tutorial-in-single-workspace/        # rCore 实验代码基座（组件化 crates）
  ai4ose-lab1-2026s/                         # 本仓库：教学环境 + 记录
```

课堂作业只需要你把 `ai4ose-lab1-2026s`（以及后续的 `ai4ose-lab2-2026s`、`ai4ose-lab3-2026s` 等）作为独立仓库推送到 `github.com/learningos`。`rCore-Tutorial-in-single-workspace` 和外层工作目录是否作为 git 仓库、如何管理，完全由你自己决定。

后续文档默认你能在 `rCore-Tutorial-in-single-workspace` 里直接执行：

- `cargo qemu --ch <n>`：在 QEMU 运行第 n 章
- `cargo qemu --ch 1 --lab`：运行 ch1-lab
- `tg-checker`：检查输出是否通过（见 `docs/5-basic-labs.md`）

## **常规浏览：安装与运行本 crate**

### 1. 安装 Rust 工具链

本项目使用 Rust 语言编写，需要安装 Rust 工具链（包含 `rustc` 编译器和 `cargo` 构建工具）。

**Linux / macOS / WSL：**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装完成后，按照提示将 Rust 加入环境变量（或重新打开终端）：

```bash
source "$HOME/.cargo/env"
```

**Windows：**

从 [https://rustup.rs](https://rustup.rs) 下载并运行 `rustup-init.exe`，按照提示完成安装。

验证安装：

```bash
rustc --version
cargo --version
```

### 2. 直接下载安装执行：显示实验内容

```bash
cargo install ai4ose-lab1-2026s
ai4ose-lab1-2026s
```

### 3. 源代码下载编译运行：显示实验内容

```bash
git clone https://github.com/learningos/ai4ose-lab1-2026s.git
cd ai4ose-lab1-2026s
cargo run
```

## **学习与验收建议（最小闭环）**

如果你现在要“快速满足 Lab1 的硬性要求（至少 5 个基础实验 + 教学环境雏形）”，建议按这个顺序：

- **先读**：`docs/START.md`
- **再做**：`docs/5-basic-labs.md` 里的 5 个实验（每个实验都要求留下可复现的命令与输出/截图）
- **最后写**：把记录放到“当周对应的 lab 仓库”（例如 lab2、lab3…）