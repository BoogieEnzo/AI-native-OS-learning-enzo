//! 第一章：应用程序与基本执行环境
//!
//! 心智模型（先有个整体印象）：  
//! 1. 裸机编程就是：CPU 上电后什么都没有，我们自己告诉它「从哪条指令开始跑、栈放在哪」。  
//! 2. 这份代码做的事只有三步：准备一块内存当栈 → 把栈指针 sp 指到那块内存 → 跳到 `rust_main` 打印字符串并通过 SBI 关机。  
//! 3. 代码里大量 `#![...]` / `#[...]` 都是**给编译器/链接器看的说明书**，控制「不使用标准库、入口放在哪个段、在什么架构下生效」等，不会直接变成 CPU 指令。
//!
//! 本章实现了一个最简单的 RISC-V S 态裸机程序，展示操作系统的最小执行环境。
#![no_std]  // crate 级属性：不使用 Rust 标准库（std），因为裸机环境没有操作系统支持，只能使用核心库（core）；这是编译期指令，不会生成机器码
#![no_main]  // crate 级属性：不生成默认的 main 入口，而是由我们自己提供入口 _start；同样只是告诉编译器怎么处理入口
#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]  // 根据条件添加属性：在 riscv64 架构下，开启严格检查（有警告就当错误、缺文档也报错）
#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]  // 在非 riscv64 架构下，允许未使用的代码存在（因为 stub 占位实现只为通过编译）

use tg_sbi;  // 导入 tg_sbi crate，这是一个提供 SBI（Supervisor Binary Interface）接口的库，用于与底层硬件交互

/// Supervisor 汇编入口。
///
/// 心智模型：CPU（或前面的固件/bootloader）最终会跳到 `_start`，这里是我们掌控的一切的起点。  
/// - 这里不做「业务逻辑」，只做两件事：1）准备好栈；2）跳到真正的 Rust 入口 `rust_main`。  
/// - 一切 `#[...]` 属性都是告诉编译器/链接器「这个函数是入口，要放在哪个段，只在 riscv64 下生效」之类的元信息，本身不会生成指令。
#[cfg(target_arch = "riscv64")]  // 条件编译：只有当目标架构是 riscv64 时才编译这个函数
#[unsafe(naked)]  // 这是一个"裸函数"（naked function），意味着编译器不会生成函数序言和尾声代码，完全由我们自己控制汇编
#[no_mangle]  // 不要改变函数名，保持为 _start，这样链接器才能找到它作为程序入口点
#[link_section = ".text.entry"]  // 将这个函数放在链接脚本的 .text.entry 段中，通常是程序的入口地址
unsafe extern "C" fn _start() -> ! {  // extern "C" 表示使用 C 调用约定；-> ! 表示这个函数永远不会返回（发散函数）
    const STACK_SIZE: usize = 4096;  // 定义栈大小为 4096 字节（4KB），const 表示编译时常量

    #[link_section = ".bss.uninit"]  // 将这个静态变量放在 .bss.uninit 段中（未初始化的数据段）
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];  // 定义一个可变的静态数组作为栈空间，类型是 u8 数组，大小为 STACK_SIZE

    core::arch::naked_asm!(  // 内联汇编宏，用于直接编写汇编代码
        "la sp, {stack} + {stack_size}",  // la 是 RISC-V 的"加载地址"指令，将栈指针 sp 设置为 STACK 的末尾（栈从高地址向低地址增长）
        "j  {main}",  // j 是跳转指令，跳转到 rust_main 函数
        stack_size = const STACK_SIZE,  // 将常量 STACK_SIZE 传递给汇编代码
        stack      =   sym STACK,  // sym 表示符号引用，将 STACK 变量的地址传递给汇编
        main       =   sym rust_main,  // 将 rust_main 函数的地址传递给汇编
    )
}

/// 非常简单的 Supervisor 裸机程序。
///
/// 打印 `Hello, World!`，然后关机。
extern "C" fn rust_main() -> ! {  // 使用 C 调用约定；-> ! 表示永不返回（程序会关机）
    for c in b"Hello, world!\n" {  // b"..." 是字节字符串字面量，返回 &[u8]；for 循环遍历每个字节
        tg_sbi::console_putchar(*c);  // *c 是解引用操作，获取字节值；console_putchar 通过 SBI 调用在控制台输出字符
    }
    tg_sbi::shutdown(false)  // 调用 SBI 关机函数，false 表示正常关机（不是错误）
}

/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]  // 这是 Rust 的 panic 处理函数属性，当程序发生 panic（不可恢复的错误）时会调用这个函数（依然只是给编译器看的标记）
fn panic(_: &core::panic::PanicInfo) -> ! {  // _ 表示忽略参数（panic 信息）；-> ! 使用了 Rust 的「never 类型」`!`，表示这个函数不会正常返回（要么关机，要么无限循环）
    tg_sbi::shutdown(true)  // true 表示异常关机（发生了错误）
}

/// 非 RISC-V64 架构的占位实现
#[cfg(not(target_arch = "riscv64"))]  // 条件编译：只有当目标架构不是 riscv64 时才编译这个模块（用于在其他架构上编译时避免链接错误）
mod stub {  // mod 定义一个模块，stub 是占位代码的意思
    #[no_mangle]  // 保持函数名不变
    pub extern "C" fn main() -> i32 {  // pub 表示公开函数；返回 i32 类型
        0  // 返回 0 表示成功
    }

    #[no_mangle]
    pub extern "C" fn __libc_start_main() -> i32 {  // 这是 C 标准库的启动函数，提供占位实现
        0
    }

    #[no_mangle]
    pub extern "C" fn rust_eh_personality() {}  // Rust 异常处理个性函数，用于异常处理，这里提供空实现
}
