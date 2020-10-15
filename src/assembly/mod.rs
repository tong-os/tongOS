global_asm!(include_str!("entry.S"));
global_asm!(include_str!("memory.S"));

extern "C" {
    pub static BSS_START: usize;
    pub static BSS_END: usize;
    pub static HEAP_START: usize;
    pub static HEAP_SIZE: usize;
    pub static TEXT_START: usize;
    pub static TEXT_END: usize;
    pub static DATA_START: usize;
    pub static DATA_END: usize;
    pub static RODATA_START: usize;
    pub static RODATA_END: usize;
    pub static KERNEL_STACK_START: usize;
    pub static KERNEL_STACK_END: usize;
}

