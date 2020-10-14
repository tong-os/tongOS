use tong_os::assignment;
use tong_os::{print, println};

extern "C" {
    static BSS_START: usize;
    static BSS_END: usize;
    #[allow(dead_code)]
    static HEAP_START: usize;
    #[allow(dead_code)]
    static HEAP_SIZE: usize;
    #[allow(dead_code)]
    static TEXT_START: usize;
    #[allow(dead_code)]
    static TEXT_END: usize;
    #[allow(dead_code)]
    static DATA_START: usize;
    #[allow(dead_code)]
    static DATA_END: usize;
    #[allow(dead_code)]
    static RODATA_START: usize;
    #[allow(dead_code)]
    static RODATA_END: usize;
    #[allow(dead_code)]
    static KERNEL_STACK_START: usize;
    #[allow(dead_code)]
    static KERNEL_STACK_END: usize;
}

extern "C" {
    fn mtvec_clint_vector_table() -> !;
}

unsafe fn setup_hart() {
    asm!("csrw mtvec, {}", in(reg) (mtvec_clint_vector_table as usize | 0x1));

    // mstatus.mie = 1
    asm!("csrw mstatus, 0b1 << 3");

    // mie.msie = 1
    asm!("csrw mie, 0b1 << 3");
}

unsafe fn software_interrupt(hartid: usize) {
    let clint_base = 0x200_0000 as *mut u32;

    clint_base.add(hartid).write_volatile(0x1);
}

#[no_mangle]
extern "C" fn kinit(hartid: usize) -> ! {
    if hartid == 0 {
        tong_os::uart::Uart::new(0x1000_0000).init();
        print!("Set all bytes in BSS to zero ...");
        for address in unsafe { BSS_START..BSS_END } {
            unsafe {
                (address as *mut usize).write_volatile(0);
            }
        }
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
        println!("Init tests!");
        assignment::print_sections();
        assignment::test_bss();
        unsafe {
            setup_hart();
            let target_hart = 2;
            println!("sending software interrupt to hart {}", target_hart);
            software_interrupt(target_hart);
        }
    } else {
        unsafe {
            setup_hart();
            loop {
                asm!("wfi");
            }
        }
    }
    loop {}
}
