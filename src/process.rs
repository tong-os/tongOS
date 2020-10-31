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
    pub ppid: Option<usize>,
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

        Process {
            context,
            stack: stack as *mut u8,
            state: ProcessState::Ready,
            page_table,
            quantum: DEFAULT_QUANTUM,
            pid,
            ppid: None,
        }
    }

    // If process has child, return it to reschedule
    pub fn schedule_child(&self) -> &'static Option<Process> {
        unsafe {
            if let Some(mut process_list) = PROCESS_LIST.take() {
                if let Some(p) = process_list.front() {
                    match p.ppid {
                        Some(pid) if pid == self.pid => {
                            let mut first = process_list.pop_front().unwrap();
                            first.state = ProcessState::Running;

                            let mut running = PROCESS_RUNNING.take().unwrap();

                            running.state = ProcessState::Ready;

                            process_list.push_front(running);

                            PROCESS_RUNNING.replace(first);

                            PROCESS_LIST.replace(process_list);
                            return &PROCESS_RUNNING;
                        }
                        Some(_) => (),
                        None => (),
                    }
                } else {
                    panic!("No more processes! Shutting down...")
                }
                PROCESS_LIST.replace(process_list);
            }
        }
        &None
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        println!("drop pid: {}", self.pid);
        page::dealloc(self.stack);
        unsafe { (*self.page_table).unmap() }
        page::dealloc(self.page_table as *mut u8);
    }
}

fn make_user_syscall(arg0: usize, arg1: usize, arg2: usize) {
    unsafe {
        // asm!("mv a0, {}", in(reg) arg0);
        // asm!("mv a1, {}", in(reg) arg1);
        // asm!("mv a2, {}", in(reg) arg2);
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
        debug!("exiting from pid: {}", PROCESS_RUNNING.as_ref().unwrap().pid);
    }
    make_user_syscall(0, 0, 0);
}

pub fn join(pid: usize) {
    // Parent process called join
    match unsafe { PROCESS_RUNNING.take() } {
        Some(process) => {
            let ppid = process.pid;
            unsafe { PROCESS_RUNNING.replace(process) };
            debug!("running pid {} join pid {}", ppid, pid);

            //set ppid of pid
            set_ppid(pid, ppid);
            exit();
        }
        None => panic!("Join called but there is no running process!"),
    }
}

pub fn set_ppid(pid: usize, ppid: usize) {
    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        for process in &mut process_list {
            if process.pid == pid {
                process.ppid = Some(ppid);
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
