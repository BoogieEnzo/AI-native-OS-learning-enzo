# AI-native OS learning (Enzo)

Local workspace for OS / rCore Tutorial learning and course labs.

## Structure

| Path | Description |
|------|-------------|
| **tg-rcore-tutorial/** | [TanGram rCore Tutorial](https://github.com/rcore-os/tg-rcore-tutorial) — componentized rCore Tutorial (ch1–ch8 + tg-* crates). Has its own git repo; sync with upstream there. |
| **tg-ch3/** | Chapter 3 （多道程序与分时多任务） |
| **tg-ch4/** | Chapter 4 （地址空间与虚拟内存） |
| **tg-ch5/** | Chapter 5 （进程管理） |
| **ai4ose-lab1-2026s/** | AI4OSE 2026 spring lab1 |
| **docs/** | 学习笔记与计划（见下表） |

### docs/ 目录

| 文件 | 说明 |
|------|------|
| **春节学习计划.md** | 春节假期学习计划（第七章、第八章） |
| **result2.11-2.17.txt** | 学习进度记录与反馈（2.11–2.17 周） |

## Git

- **upstream:** `rcore-os/tg-rcore-tutorial` (for reference; actual tutorial code lives in `tg-rcore-tutorial/` as a nested repo)

To sync the tutorial with upstream, run `git fetch` and `git merge` inside `tg-rcore-tutorial/`, not in this root.

## Quick start

- **Run a chapter (e.g. ch3):** `cd tg-rcore-tutorial/ch3 && cargo run`
- **Labs:** see `ai4ose-lab1-2026s/README.md`
