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
pub static DEFAULT_QUANTUM: usize = 1;
// Process States
// Tanenbaum, Modern Operating Systems
// Ready -> Running = picked by scheduler
// Running -> Ready = scheduler picks another process
// Running -> Blocked = blocked for input
// Running -> Sleeping = process sleep
// Blocked -> Ready = input available now
// Sleeping -> Running/Ready = wake up
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Sleeping,
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
    pub blocking_pid: Option<usize>,
    pub sleep_until: usize,
}

impl Process {
    pub fn new(start: usize, arg0: usize) -> Self {
        // cpu::disable_global_interrupts();
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

        let num_stack_pages = 8;
        let stack = page::zalloc(num_stack_pages) as usize;
        let stack_end = stack + num_stack_pages * page::PAGE_SIZE;

        context.regs[cpu::GeneralPurposeRegister::Sp as usize] = stack_end;

        let page_table = page_table_address as *mut Sv39PageTable;

        unsafe {
            for address in (stack..stack_end).step_by(page::PAGE_SIZE) {
                (*page_table).map(
                    address,
                    address,
                    PageTableEntryFlags::UserReadWrite as usize,
                    0,
                );
            }
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
            for address in (assembly::DATA_START..assembly::DATA_END).step_by(page::PAGE_SIZE) {
                (*page_table).map(
                    address as usize,
                    address as usize,
                    PageTableEntryFlags::UserReadWrite as usize,
                    0,
                );
            }
            for address in (assembly::BSS_START..assembly::BSS_END).step_by(page::PAGE_SIZE) {
                (*page_table).map(
                    address as usize,
                    address as usize,
                    PageTableEntryFlags::UserReadWrite as usize,
                    0,
                );
            }
            for address in (assembly::HEAP_START..assembly::HEAP_START + assembly::HEAP_SIZE)
                .step_by(page::PAGE_SIZE)
            {
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

        let proc = Process {
            context,
            stack: stack as *mut u8,
            state: ProcessState::Ready,
            page_table,
            quantum: DEFAULT_QUANTUM,
            pid,
            blocking_pid: None,
            sleep_until: 0,
        };
        // cpu::enable_global_interrupts();
        proc
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        debug!("drop pid: {}", self.pid);
        page::dealloc(self.stack);
        unsafe { (*self.page_table).unmap() }
        page::dealloc(self.page_table as *mut u8);
    }
}

fn make_user_syscall(_arg0: usize, _arg1: usize, _arg2: usize) {
    unsafe {
        asm!("ECALL");
    }
}

pub fn create_thread(func: usize, arg0: usize) -> usize {
    make_user_syscall(1, func, arg0);
    let pid: usize;
    unsafe {
        asm!("mv {}, a0", out(reg) pid);
    }
    pid
}

pub fn exit() {
    unsafe {
        debug!(
            "exiting from pid: {}",
            PROCESS_RUNNING.as_ref().unwrap().pid
        );
    }
    make_user_syscall(0, 0, 0);
}

pub fn join(pid: usize) {
    unsafe {
        debug!(
            "Join running.pid: {}, waiting pid {}",
            PROCESS_RUNNING.as_ref().unwrap().pid,
            pid
        );
    }
    make_user_syscall(2, pid, 0);
}

pub fn sleep(amount: usize) {
    make_user_syscall(3, amount, 0);
}

pub fn read_line(buffer: &mut alloc::string::String) {
    make_user_syscall(4, buffer as *mut _ as usize, 0);
    while unsafe { crate::uart::READING } {}
}

pub fn set_blocking_pid(pid: usize, blocking_pid: usize) {
    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        for process in &mut process_list {
            if process.pid == pid {
                process.blocking_pid = Some(blocking_pid);
            }
        }
        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }
}

pub fn wake_process(blocked_pid: usize) {
    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        for process in &mut process_list {
            if process.pid == blocked_pid {
                process.state = ProcessState::Ready;
            }
        }
        unsafe {
            PROCESS_LIST.replace(process_list);
        }
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

pub fn process_list_contains(pid: usize) -> bool {
    let mut contains = false;
    if let Some(process_list) = unsafe { PROCESS_LIST.take() } {
        if let Some(_) = process_list.iter().position(|process| process.pid == pid) {
            contains = true;
        }

        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }
    contains
}

pub fn switch_to_user(process: &Process) -> ! {
    debug!("switch_to_user: {}", process.pid);
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
