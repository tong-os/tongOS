// lock.rs
// Locking routines
// Stephen Marz
// tongOS team

#[repr(u32)]
#[derive(Clone, Copy)]
enum MutexState {
    Unlocked = 0,
    #[allow(dead_code)]
    Locked = 1,
}

#[derive(Clone, Copy)]
pub struct Mutex {
    #[allow(dead_code)]
    state: MutexState,
}

impl<'a> Mutex {

    pub const fn new() -> Self {
		Self { state: MutexState::Unlocked }
    }

    // Try to lock the Mutex. True if acquired, false otherwise.
    pub fn try_lock(&mut self) -> bool {
        unsafe {
            let state: u32;
            // atomically load a 32-bit signed data value from the address in rs1,
            // place the value into register rd, swap the loaded value and the original 32-bit signed value in rs2,
            // then store the result back to the address in rs1.
            // https://msyksphinz-self.github.io/riscv-isadoc/html/rva.html#amoswap-w
            // llvm_asm!("amoswap.w.aq $0, $1, ($2)\n" : "=r"(state) : "r"(1), "r"(self) :: "volatile");
            asm!("amoswap.w.aq {}, {}, ({})", out(reg) state, in(reg) 1, in(reg) self);
            match core::mem::transmute(state) {
                MutexState::Locked => false,
                MutexState::Unlocked => true,
            }
        }
    }

    pub fn spin_lock(&mut self) {
        while !self.try_lock() {}
    }

    pub fn unlock(&mut self) {
        unsafe {
            asm!("amoswap.w.aq zero, zero, ({})", in(reg) self);
        }
    }
}
