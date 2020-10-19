#![no_std]
#![feature(allocator_api)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

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

pub mod assembly;
pub mod assignment;
pub mod kmem;
pub mod page;
pub mod process;
pub mod uart;
pub mod cpu;
