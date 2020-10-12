#![no_main]
#![no_std]
#![feature(
    panic_info_message,
    llvm_asm,
    global_asm,
    allocator_api,
    alloc_error_handler,
    const_raw_ptr_to_usize_cast,
    lang_items,
    custom_test_frameworks
)]
#![test_runner(crate::test_runner)]

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

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            llvm_asm!("wfi"::::"volatile");
        }
    }
}

mod kinit;
mod assembly;
mod uart;
