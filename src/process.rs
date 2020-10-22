// process.rs
// Kernel and user processes
// Stephen Marz
// tongOS team

use crate::{
    assembly,
    cpu::{self, TrapFrame},
    page::{self, PageTableEntryFlags, Sv39PageTable},
};

use alloc::collections::vec_deque::VecDeque;

pub static mut RRUNNING: Option<Process> = None;
pub static mut PROCESS_LIST: Option<VecDeque<Process>> = None;
pub static mut NEXT_PID: usize = 0;
pub static DEFAULT_QUANTUM: usize = 666;
// Process States
// Tanenbaum, Modern Operating Systems
// Ready -> Running = picked by scheduler
// Running -> Ready = scheduler picks another process
// Running -> Blocked = blocked for input
// Blocked -> Ready = input available now
#[derive(Debug, Clone, Copy)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Process {
    pub context: TrapFrame,
    pub stack: *mut u8,
    pub state: ProcessState,
    pub page_table: *mut Sv39PageTable,
    pub quantum: usize,
    pub pid: usize,
}

impl Process {
    pub fn new(start: fn() -> ()) -> Self {
        let mut context = TrapFrame::new();
        context.pc = start as usize;
        context.mode = cpu::CpuMode::Machine as usize;

        let pid = unsafe {
            NEXT_PID += 1;
            NEXT_PID
        };

        let page_table_address = page::zalloc(1);

        context.satp = cpu::build_satp(pid, page_table_address as usize);

        let stack = page::zalloc(1) as usize;

        context.regs[cpu::GeneralPurposeRegister::Sp as usize] = stack + page::PAGE_SIZE;

        let page_table = page_table_address as *mut Sv39PageTable;

        unsafe {
            (*page_table).map(
                stack + page::PAGE_SIZE,
                stack + page::PAGE_SIZE,
                PageTableEntryFlags::UserReadWriteExecute as usize,
                0,
            );
            (*page_table).map(
                0x1000_0000,
                0x1000_0000,
                PageTableEntryFlags::UserReadWrite as usize,
                0,
            );
            for address in (assembly::TEXT_START..assembly::TEXT_END).step_by(1000) {
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
            quantum: DEFAULT_QUANTUM,
            pid,
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        page::dealloc(self.stack);
        unsafe { (*self.page_table).unmap() }
        page::dealloc(self.page_table as *mut u8);
    }
}

pub fn process_list_add(process: Process) {
    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        process_list.push_back(process);

        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    } else {
        let mut process_list = VecDeque::new();

        process_list.push_back(process);

        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }
}

pub fn process_list_remove(pid: usize) {
    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        if let Some(position) = process_list.iter().position(|process| process.pid == pid) {
            process_list.remove(position);
        }

        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }
}

pub fn exit() {
    unsafe { asm!("ECALL") };
}

pub fn switch_to_user(process: &Process) -> ! {
    unsafe {
        crate::assembly::__tong_os_switch_to_user(
            &process.context,
            process.context.pc,
            process.context.satp,
        );
    }
}

pub fn print_process_list() {
    if let Some(process_list) = unsafe { PROCESS_LIST.take() } {
        for process in &process_list {
            println!("Pid: {}", process.pid);
        }
        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }
}
