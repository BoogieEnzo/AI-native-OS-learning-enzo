# AI4OSE Lab1（从这里开始）

## 1. 你有两个仓库

1) 实验代码：`../rCore-Tutorial-in-single-workspace/`

2) 本仓库：`ai4ose-lab1-2026s/`（只放说明/路线，不再放周报和 AI 记录）

> 你已经决定：**每周一个独立仓库**（例如 lab2、lab3…），所以本仓库不维护 `weekly/ai/logs`。

在你当前的机器上，这两个仓库通常放在一个上层“工作目录”里，例如：

```text
~/AI-native-OS-learning-enzo/
  ai4ose-lab1-2026s/                   # 本仓库：说明 / 路线（以及本周的记录）
  rCore-Tutorial-in-single-workspace/  # rCore 实验代码
```

> 外层的 `AI-native-OS-learning-enzo/` 也可以是一个单独的 git 仓库，但课堂作业只要求你把每周的 `ai4ose-labX-2026s` 仓库推送到 `github.com/learningos`。

## 2. Linux 环境准备（够用版）

```bash
sudo apt-get update
sudo apt-get install -y qemu-system-misc build-essential python3 git curl
```

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add riscv64gc-unknown-none-elf
rustup component add rust-src llvm-tools-preview
```

## 3. 先跑通 ch1（最小验证）

```bash
cd ../rCore-Tutorial-in-single-workspace/ch1
cargo run
```

能看到 `Hello, world!` 并正常关机就算通过。

## 4. 5 周计划（每周 1 个 chapter）

- Week1：ch1
- Week2：ch2
- Week3：ch3
- Week4：ch4
- Week5：ch5

本仓库保留 `docs/5-basic-labs.md` 作为“ch1~ch5 路线与验收命令”的汇总。

## 5. 记录放哪里

每周的命令、输出证据、总结、以及 AI 对话整理，放到**当周对应的 lab 仓库**里（例如 lab2、lab3…）。

