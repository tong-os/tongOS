// process.rs
// Kernel and user processes
// Stephen Marz
// tongOS team

use crate::cpu::{self, TrapFrame};
use crate::page::{self, PageTableEntryFlags, Sv39PageTable};

// Process States
// Tanenbaum, Modern Operating Systems
// Ready -> Running = picked by scheduler
// Running -> Ready = scheduler picks another process
// Running -> Blocked = blocked for input
// Blocked -> Ready = input available now
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
}

pub struct Process {
    pub context: TrapFrame,
    pub stack: *mut u8,
    pub state: ProcessState,
    pub page_table: *mut Sv39PageTable,
}

impl Process {
    pub fn new(start: fn() -> ()) -> Self {
        let mut context = TrapFrame::new();
        context.pc = start as usize;
        context.mode = cpu::CpuMode::Machine as usize;

        let page_table_address = crate::page::zalloc(1);

        context.satp = crate::cpu::build_satp(1, page_table_address as usize);

        let stack = crate::page::zalloc(1) as usize;

        context.regs[crate::cpu::GeneralPurposeRegister::Sp as usize] =
            stack + crate::page::PAGE_SIZE;

        let page_table = page_table_address as *mut Sv39PageTable;

        unsafe {
            (*page_table).map(
                stack + crate::page::PAGE_SIZE,
                stack + crate::page::PAGE_SIZE,
                PageTableEntryFlags::UserReadWriteExecute as usize,
                0,
            );
            (*page_table).map(
                0x1000_0000,
                0x1000_0000,
                PageTableEntryFlags::UserReadWrite as usize,
                0,
            );
            for address in (crate::assembly::TEXT_START..crate::assembly::TEXT_END).step_by(1000) {
                (*page_table).map(
                    address as usize,
                    address as usize,
                    PageTableEntryFlags::UserReadExecute as usize,
                    0,
                );
            }
        }

        Process {
            context,
            stack: stack as *mut u8,
            state: ProcessState::Ready,
            page_table,
        }
    }
}

pub fn switch_to_user(trap_frame: &TrapFrame) -> ! {
    unsafe {
        println!(
            "pc {:x?} sattp: {:x?},  pc: {:x?}",
            trap_frame.pc, trap_frame.satp, trap_frame.regs[2]
        );
        crate::assembly::__tong_os_switch_to_user(trap_frame, trap_frame.pc, trap_frame.satp);
    }
}
