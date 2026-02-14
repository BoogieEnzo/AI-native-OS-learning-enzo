# AI-native OS learning (Enzo)

Local workspace for OS / rCore Tutorial learning and course labs.

## Structure

| Path | Description |
|------|-------------|
| **tg-rcore-tutorial/** | [TanGram rCore Tutorial](https://github.com/rcore-os/tg-rcore-tutorial) — componentized rCore Tutorial (ch1–ch8 + tg-* crates). Has its own git repo; sync with upstream there. |
| **ai4ose-lab1-2026s/** | AI4OSE 2026 spring lab1. |

## Git

- **origin:** your fork (`boogieenzo/AI-native-OS-learning-enzo`)
- **upstream:** `rcore-os/tg-rcore-tutorial` (for reference; actual tutorial code lives in `tg-rcore-tutorial/` as a nested repo)

To sync the tutorial with upstream, run `git fetch` and `git merge` inside `tg-rcore-tutorial/`, not in this root.

## Quick start

- **Run a chapter (e.g. ch3):** `cd tg-rcore-tutorial/ch3 && cargo run`
- **Labs:** see `ai4ose-lab1-2026s/README.md`
