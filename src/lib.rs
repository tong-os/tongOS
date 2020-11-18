#![no_std]
#![feature(allocator_api)]
#![feature(alloc_prelude)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate alloc;

// 1 = Simple example processess.
// 2 = Philosopher's Dinner;
// 3 = Keyboard input app example.
// 4 = All processess.
pub const PROCESS_TO_RUN: usize = 2;

pub const DEBUG_OUTPUT: bool = false;
pub const ENABLE_PREEMPTION: bool = true;

pub static mut KPRINT_LOCK: crate::lock::Mutex = crate::lock::Mutex::new();

pub fn get_print_lock() -> &'static mut crate::lock::Mutex {
    unsafe { &mut KPRINT_LOCK }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

pub fn _print(args: core::fmt::Arguments) {
    get_print_lock().spin_lock();
    let mut uart = uart::Uart::new(0x1000_0000);
    use core::fmt::Write;
    uart.write_fmt(args).unwrap();
    get_print_lock().unlock();
}

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
        $crate::_print(format_args!($($args)+))
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        print!("\r\n")
    }};
    ($fmt:expr) => {{
        print!(concat!($fmt, "\r\n"))
    }};
    ($fmt:expr, $($args:tt)+) => {{
        print!(concat!($fmt, "\r\n"), $($args)+)
    }};
}

#[macro_export]
macro_rules! debug {
    () => {{
        if $crate::DEBUG_OUTPUT {
            println!(concat!("hart {}: "), $crate::cpu::get_mhartid())
        }
    }};
    ($fmt:expr) => {{
        if $crate::DEBUG_OUTPUT {
            println!(concat!("hart {}: ", $fmt), $crate::cpu::get_mhartid())
        }
    }};
    ($fmt:expr, $($args:tt)+) => {{
        if $crate::DEBUG_OUTPUT {
            println!(concat!("hart {}: ", $fmt), $crate::cpu::get_mhartid(), $($args)+)
        }
    }};
}

pub mod app;
pub mod assembly;
pub mod assignment;
pub mod cpu;
pub mod kmem;
pub mod lock;
pub mod page;
pub mod plic;
pub mod process;
pub mod scheduler;
pub mod trap;
pub mod uart;
