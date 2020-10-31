#![no_std]
#![feature(allocator_api)]
#![feature(alloc_prelude)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#[macro_use]
extern crate alloc;

use alloc::prelude::v1::*;

pub const DEBUG_OUTPUT: bool = true;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
        let mut uart = $crate::uart::Uart::new(0x1000_0000);
        use core::fmt::Write;
        let _ = write!(uart, $($args)+);
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
        if crate::DEBUG_OUTPUT {
            println!()
        }
    }};
    ($fmt:expr) => {{
        if crate::DEBUG_OUTPUT {
            println!($fmt)
        }
    }};
    ($fmt:expr, $($args:tt)+) => {{
        if crate::DEBUG_OUTPUT {
            println!($fmt, $($args)+)
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
pub mod process;
pub mod scheduler;
pub mod trap;
pub mod uart;
