extern "C" {
    static BSS_START: usize;
    static BSS_END: usize;
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
    static TEXT_START: usize;
    static TEXT_END: usize;
    static DATA_START: usize;
    static DATA_END: usize;
    static RODATA_START: usize;
    static RODATA_END: usize;
    static KERNEL_STACK_START: usize;
    static KERNEL_STACK_END: usize;
}

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
                "BSS section contains non-zero value at address: 0x{:x}. Value={}",
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