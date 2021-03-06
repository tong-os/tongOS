use crate::assembly::*;
use crate::process;
use alloc::format;

pub fn test_bss() {
    print!("Checking BSS ...  ");
    let mut non_zeroes_count = 0;
    for address in unsafe { BSS_START..BSS_END } {
        let content = unsafe { (address as *mut usize).read_volatile() };
        if content != 0_usize {
            if non_zeroes_count == 0 {
                println!();
            }
            non_zeroes_count += 1;
            println!(
                "BSS section contains non-zero value at address: 0x{:x?}. Value={:x?}",
                address, content
            );
        }
    }
    if non_zeroes_count > 0 {
        panic!("Error: BSS check Failed!")
    }

    println!("Ok!");
}

pub fn print_sections() {
    unsafe {
        println!("TEXT         {:#8x} ~ {:#8x}", TEXT_START, TEXT_END);
        println!("RODATA       {:#8x} ~ {:#8x}", RODATA_START, RODATA_END);
        println!("DATA         {:#8x} ~ {:#8x}", DATA_START, DATA_END);
        println!("BSS          {:#8x} ~ {:#8x}", BSS_START, BSS_END);
        println!(
            "KERNEL STACK {:#8x} ~ {:#8x}",
            KERNEL_STACK_START, KERNEL_STACK_END
        );
        println!(
            "HEAP         {:#8x} ~ {:#8x}",
            HEAP_START,
            HEAP_START + HEAP_SIZE
        );
    }
}

pub fn example_process1(test: usize) -> () {
    process::print_str("Example process 1");
    process::print_str("YEAH, we're running as user with virtual address translation!");

    // some_math(100);

    process::print_str(&format!("Arg: {}", test));

    process::print_str("exiting process");
    process::exit();
}

pub fn example_process2() -> () {
    process::print_str("EXAMPLE 2, ARE YOU READY??");

    // some_math(100);

    process::print_str("exiting process");
    process::exit();
}

pub fn example_process3(iteration: usize) {
    process::print_str("Example process 3!");

    // some_math(100);

    process::print_str(&format!("Counting for {}", iteration));
    let mut my_counter = 0;
    for _ in 0..iteration {
        my_counter += 1;
    }
    process::print_str(&format!(
        "Ex3 counter = {}. Expected = {}",
        my_counter, iteration
    ));
    process::exit();
}

pub fn choose_processes(process_to_run: usize) {
    match process_to_run {
        1 => {
            let process = process::Process::new(example_process1 as usize, 666, 0, 0);
            process::process_list_add(process);
            let process = process::Process::new(example_process2 as usize, 0, 0, 0);
            process::process_list_add(process);
            let process = process::Process::new(example_process3 as usize, 666, 0, 0);
            process::process_list_add(process);
            let process = process::Process::new(example_process3 as usize, 42, 0, 0);
            process::process_list_add(process);
        }
        2 => {
            let process = process::Process::new(crate::app::philosopher::main as usize, 0, 0, 0);
            process::process_list_add(process);
        }
        3 => {
            let process = process::Process::new(crate::app::input_example::main as usize, 0, 0, 0);
            process::process_list_add(process);
        }
        4 => {
            choose_processes(1);
            choose_processes(3);
            choose_processes(2);
        }
        _ => {
            println!("Process not found!");
        }
    }
}
