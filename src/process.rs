// process.rs
// Kernel and user processes
// Stephen Marz
// tongOS team

use crate::assembly;
use crate::cpu::{self, CpuMode, TrapFrame};
use crate::page::{self, PageTableEntryFlags, Sv39PageTable};

use alloc::collections::vec_deque::VecDeque;

pub static mut PROCESS_RUNNING: Option<Process> = None;
pub static mut PROCESS_LIST: Option<VecDeque<Process>> = None;
pub static mut NEXT_PID: usize = 0;
pub static DEFAULT_QUANTUM: usize = 666;
// Process States
// Tanenbaum, Modern Operating Systems
// Ready -> Running = picked by scheduler
// Running -> Ready = scheduler picks another process
// Running -> Blocked = blocked for input
// Blocked -> Ready = input available now
#[repr(C)]
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
    pub fn new(start: usize, arg0: usize) -> Self {
        let mut context = TrapFrame::new();
        context.pc = start as usize;
        context.mode = CpuMode::User as usize;

        context.regs[cpu::GeneralPurposeRegister::A0 as usize] = arg0;

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
            (*page_table).map(stack, stack, PageTableEntryFlags::UserReadWrite as usize, 0);
            (*page_table).map(
                0x1000_0000,
                0x1000_0000,
                PageTableEntryFlags::UserReadWrite as usize,
                0,
            );
            for address in (assembly::RODATA_START..assembly::RODATA_END).step_by(page::PAGE_SIZE) {
                (*page_table).map(
                    address as usize,
                    address as usize,
                    PageTableEntryFlags::UserReadWrite as usize,
                    0,
                );
            }
            for address in (assembly::TEXT_START..assembly::TEXT_END).step_by(page::PAGE_SIZE) {
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

    pub fn join(&mut self) {
        match unsafe { PROCESS_RUNNING.take() } {
            Some(mut process) => {
                let pid = process.pid;
                process.state = ProcessState::Ready;
                process_list_add(process);
                match crate::scheduler::schedule() {
                    Some(next) => {
                        switch_to_user(next);
                    }
                    None => {
                        panic!("Couldn't join! Process pid={}", pid);
                    }
                }
            }
            None => {
                panic!("Join called but there is no running process!")
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
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
    unsafe { assembly::__tong_os_switch_to_user(process) }
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
