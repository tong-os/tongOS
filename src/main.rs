#![no_main]
#![no_std]
#![feature(
    panic_info_message,
    asm,
    global_asm,
    allocator_api,
    alloc_prelude,
    alloc_error_handler,
    lang_items,
    custom_test_frameworks
)]
#![test_runner(crate::test_runner)]

use tong_os::{print, println};

#[macro_use]
extern crate alloc;

use alloc::prelude::v1::*;

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
            asm!("wfi");
        }
    }
}

use tong_os::assembly::*;
use tong_os::assignment;

fn init() {
    println!("YEAH, we're running as user with virtual address translation!");

    loop {}
}

#[no_mangle]
extern "C" fn kinit(_hartid: usize) -> ! {
    tong_os::uart::Uart::new(0x1000_0000).init();
    print!("Set all bytes in BSS to zero ...");
    for address in unsafe { BSS_START..BSS_END } {
        unsafe {
            (address as *mut usize).write_volatile(0);
        }
    }
    println!("Finished!");
    println!("Init tests!");
    assignment::print_sections();
    assignment::test_bss();

    println!("Init pages");
    tong_os::page::init();
    tong_os::page::print_page_allocations();
    tong_os::kmem::init();
    tong_os::kmem::print_table();
    let x = vec![0, 1, 2, 3];
    tong_os::kmem::print_table();

    println!("Finished!");

    println!("You are now in ...");
    println!(concat!(
        " _                    _____ _____ \n",
        "| |                  |  _  /  ___|\n",
        "| |_ ___  _ __   __ _| | | \\ `--. \n",
        "| __/ _ \\| '_ \\ / _` | | | |`--. \\\n",
        "| || (_) | | | | (_| \\ \\_/ /\\__/ /\n",
        " \\__\\___/|_| |_|\\__, |\\___/\\____/ \n",
        "                 __/ | \n",
        "                |___/ ",
    ));

    println!("Before init: {:x?}", init as usize);
    let process = tong_os::process::Process::new(init);
    println!("after init");

    println!("Before swtich");
    tong_os::process::switch_to_user(&process.context);
}
