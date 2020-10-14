global_asm!(include_str!("trap.S"));

#[no_mangle]
extern "C" fn trap_handler() {
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
        if cause >> 63 & 1 == 1 { "async" } else { "sync"},
        hartid,
        cause & 0xfff
    );

    loop {};
}

#[no_mangle]
pub extern "C" fn machine_software_interrupt() {
    let hartid: usize;
    unsafe {
        asm!("csrr {}, mhartid", out(reg) hartid);
    }

    println!(
        "handling software interrupt on hart {}",
        hartid
    );

    loop {};
}
