// cpu.rs
// CPU and CPU-related routines
// Also contains the kernel's trap frame
// Stephen Marz
// tongOS team

// The frequency of QEMU is 10 MHz
pub const FREQ: u64 = 10_000_000;
// Let's do this 250 times per second for switching
pub const CONTEXT_SWITCH_TIME: u64 = FREQ / 500;

#[repr(usize)]
pub enum CpuMode {
    User = 0b00,
    Supervisor = 0b01,
    Machine = 0b11,
}

#[repr(usize)]
pub enum GeneralPurposeRegister {
    Zero = 0,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
}

#[repr(usize)]
pub enum FloatingPointRegister {
    Ft0 = 0,
    Ft1,
    Ft2,
    Ft3,
    Ft4,
    Ft5,
    Ft6,
    Ft7,
    Fs0,
    Fs1,
    Fa0,
    Fa1,
    Fa2,
    Fa3,
    Fa4,
    Fa5,
    Fa6,
    Fa7,
    Fs2,
    Fs3,
    Fs4,
    Fs5,
    Fs6,
    Fs7,
    Fs8,
    Fs9,
    Fs10,
    Fs11,
    Ft8,
    Ft9,
    Ft10,
    Ft11,
}
// This allows for quick reference and full
// context switch handling.
// To make offsets easier, everything will be a usize (8 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub satp: usize,
    pub pc: usize,
    pub global_interrupt_enable: usize,
    pub mode: usize,
}

impl TrapFrame {
    pub fn new() -> Self {
        TrapFrame {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            pc: 0,
            global_interrupt_enable: 0,
            mode: 0,
        }
    }
}
// SATP = MODE |  ASID  |  PPN
//      [63:60]|[59:44] | [43:0]
pub const fn build_satp(asid: usize, pysical_address: usize) -> usize {
    use crate::page;
    let mode = (page::Sv39PageTable::mode() as usize) << 60;
    let asid = (asid & 0xffff) << 44;
    let ppn_mask = (1 << 44) - 1;
    let pysical_page_number = pysical_address >> page::PAGE_ORDER & ppn_mask;

    mode | asid | pysical_page_number
}

pub fn disable_global_interrupts() {
    debug!("Disable global interrupts for hart {}!", get_mhartid());
    unsafe {
        let mstatus: usize;
        asm!("csrr {}, mstatus", out(reg) mstatus);
        let mstatus = mstatus & !(1 << 3);
        asm!("csrw mstatus, {}", in(reg) mstatus);
    }
}

pub fn enable_global_interrupts() {
    debug!("Enable global interrupts for hart {}!", get_mhartid());
    unsafe {
        // let mstatus: usize;
        // asm!("csrr {}, mstatus", out(reg) mstatus);
        // [3] = MIE (Machine Interrupt Enable)
        let mstatus = 1 << 3;
        asm!("csrw mstatus, {}", in(reg) mstatus);
    }
}

pub fn get_mhartid() -> usize {
    unsafe {
        let hartid: usize;
        asm!("csrr {}, mhartid", out(reg) hartid);
        hartid
    }
}

pub fn get_mcause() -> usize {
    let mcause: usize;
    unsafe { asm!("csrr {}, mcause", out(reg) mcause) };
    mcause
}
