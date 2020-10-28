use crate::assembly::*;

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
