global_asm!(include_str!("trap.S"));

#[no_mangle]
extern "C" fn trap_handler() {
    println!("hit");
    let hartid: usize;
    unsafe {
        asm!("csrr {}, mhartid", out(reg) hartid);
    }

    let cause: usize;
    unsafe {
        asm!("csrr {}, mcause", out(reg) cause);
    }
    println!(
        "interrupt {} on hart {} cause {}",
        cause >> 63 & 1,
        hartid,
        cause & 0xfff
    );

    loop {}
}

#[no_mangle]
extern "C" fn machine_software_interrupt() {
    println!("hit");
    let hartid: usize;
    unsafe {
        asm!("csrr {}, mhartid", out(reg) hartid);
    }

    let cause: usize;
    unsafe {
        asm!("csrr {}, mcause", out(reg) cause);
    }
    println!(
        "software interrupt on hart {} cause {}",
        hartid,
        cause & 0xfff
    );

    loop {}
}
