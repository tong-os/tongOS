// cpu.rs
// CPU and CPU-related routines
// Also contains the kernel's trap frame
// Stephen Marz
// tongOS team

#[repr(usize)]
pub enum CpuMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
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
    pub regs: [usize; 32],  // 0 - 255
    pub fregs: [usize; 32], // 256 - 511
    pub satp: usize,        // 512 - 519
    pub pc: usize,          // 520
    pub hartid: usize,      // 528
    pub mode: usize,        // 552
}

impl TrapFrame {
    pub fn new() -> Self {
        TrapFrame {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            pc: 0,
            hartid: 0,
            mode: 0,
        }
    }
}
// SATP = MODE |  ASID  |  PPN
//      [63:60]|[59:44] | [43:0]
pub const fn build_satp(asid: usize, addr: usize) -> usize {
    (crate::page::Sv39PageTable::mode() as usize) << 60
        | (asid & 0xffff) << 44
        | (addr >> 12) & 0xff_ffff_ffff
}
