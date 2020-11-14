// process.rs
// Kernel and user processes
// Stephen Marz
// tongOS team

use crate::assembly;
use crate::cpu::{self, CpuMode, TrapFrame};
use crate::lock::Mutex;
use crate::page::{self, PageTableEntryFlags, Sv39PageTable};

use alloc::collections::vec_deque::VecDeque;

pub static mut PROCESS_LIST: Option<VecDeque<Process>> = None;
pub static mut PROCESS_LIST_LOCK: Mutex = Mutex::new();

pub static mut NEXT_PID: usize = 0;
pub static mut NEXT_PID_LOCK: Mutex = Mutex::new();
pub static DEFAULT_QUANTUM: usize = 1;

pub static mut PROCESS_LOCK: Mutex = Mutex::new();

pub static mut PROCESS_IDLE: [Option<Process>; 4] = [None, None, None, None];

pub fn get_process_lock() -> &'static mut Mutex {
    unsafe { &mut PROCESS_LOCK }
}

pub fn get_process_list_lock() -> &'static mut Mutex {
    unsafe { &mut PROCESS_LIST_LOCK }
}

fn get_next_pid_lock() -> &'static mut Mutex {
    unsafe { &mut NEXT_PID_LOCK }
}

pub fn init() {
    for process in unsafe { &mut PROCESS_IDLE } {
        process.replace(Process::new_idle());
    }
}

// Process States
// Tanenbaum, Modern Operating Systems
// Ready -> Running = picked by scheduler
// Running -> Ready = scheduler picks another process
// Running -> Blocked = blocked for input
// Running -> Sleeping = process sleep
// Blocked -> Ready = input available now
// Sleeping -> Running/Ready = wake up
#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ProcessState {
    Ready,
    Running(usize),
    Blocked,
    Sleeping(usize),
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Process {
    pub trap_frame: *mut TrapFrame,
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
        get_process_lock().spin_lock();

        let pid = unsafe {
            NEXT_PID += 1;
            NEXT_PID
        };

        let page_table_address = page::zalloc(1);

        let mut context = TrapFrame::new();
        context.regs[cpu::GeneralPurposeRegister::A0 as usize] = arg0;
        context.satp = cpu::build_satp(pid, page_table_address as usize);
        context.pc = start as usize;
        context.mode = CpuMode::User as usize;

        let num_stack_pages = 8;
        let stack = page::zalloc(num_stack_pages) as usize;
        let stack_end = stack + num_stack_pages * page::PAGE_SIZE;

        context.regs[cpu::GeneralPurposeRegister::Sp as usize] =
            stack_end - core::mem::size_of::<TrapFrame>();

        let trap_frame = context.regs[cpu::GeneralPurposeRegister::Sp as usize] as *mut TrapFrame;

        unsafe {
            let source = &context as *const TrapFrame;
            core::ptr::copy(source, trap_frame, 1);
        }

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
            trap_frame,
            stack: stack as *mut u8,
            state: ProcessState::Ready,
            page_table,
            quantum: DEFAULT_QUANTUM,
            pid,
            blocking_pid: None,
            sleep_until: 0,
        };
        get_process_lock().unlock();
        // cpu::enable_global_interrupts();
        proc
    }

    pub fn new_idle() -> Self {
        get_process_lock().spin_lock();
        let mut context = TrapFrame::new();
        context.pc = self::idle as usize;
        context.mode = CpuMode::Machine as usize;
        context.global_interrupt_enable = 1 << 7;

        let num_stack_pages = 2;
        let stack = page::zalloc(num_stack_pages) as usize;
        let stack_end = stack + num_stack_pages * page::PAGE_SIZE;

        context.regs[cpu::GeneralPurposeRegister::Sp as usize] =
            stack_end - core::mem::size_of::<TrapFrame>();

        let trap_frame = context.regs[cpu::GeneralPurposeRegister::Sp as usize] as *mut TrapFrame;

        unsafe {
            let source = &context as *const TrapFrame;
            core::ptr::copy(source, trap_frame, 1);
        }

        let proc = Process {
            trap_frame,
            stack: stack as *mut u8,
            state: ProcessState::Ready,
            page_table: 0 as *mut _,
            quantum: DEFAULT_QUANTUM,
            pid: core::usize::MAX,
            blocking_pid: None,
            sleep_until: 0,
        };

        get_process_lock().unlock();

        proc
    }

    pub fn get_trap_frame(&self) -> *mut TrapFrame {
        self.trap_frame
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
    make_user_syscall(0, 0, 0);
}

pub fn join(pid: usize) {
    make_user_syscall(2, pid, 0);
}

pub fn sleep(amount: usize) {
    make_user_syscall(3, amount, 0);
}

pub fn read_line(buffer: &mut alloc::string::String) {
    make_user_syscall(4, buffer as *mut _ as usize, 0);
    while unsafe { crate::uart::READING } {}
}

pub fn print_line(buffer: &alloc::string::String) {
    make_user_syscall(5, buffer as *const _ as usize, 0);
}

pub fn set_blocking_pid(pid: usize, blocking_pid: usize) {
    get_process_list_lock().spin_lock();
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
    get_process_list_lock().unlock();
}

pub fn wake_process(blocked_pid: usize) {
    get_process_list_lock().spin_lock();
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
    get_process_list_lock().unlock();
}

pub fn process_list_add(process: Process) {
    get_process_list_lock().spin_lock();
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
    get_process_list_lock().unlock();
}

pub fn process_list_remove(pid: usize) {
    get_process_list_lock().spin_lock();
    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        if let Some(position) = process_list.iter().position(|process| process.pid == pid) {
            process_list.remove(position);
        }

        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }
    get_process_list_lock().unlock();
}

pub fn process_list_contains(pid: usize) -> bool {
    let mut contains = false;
    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if pid == process.pid {
            contains = true;
        }
    }

    get_process_list_lock().unlock();
    contains
}

pub fn update_running_process_trap_frame(trap_frame: *mut TrapFrame) {
    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if process.state == ProcessState::Running(cpu::get_mhartid()) {
            process.trap_frame = trap_frame;
        }
    }

    get_process_list_lock().unlock();
}

pub fn get_running_process_pid() -> Option<usize> {
    let mut pid = None;

    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if process.state == ProcessState::Running(cpu::get_mhartid()) {
            pid = Some(process.pid);
        }
    }

    get_process_list_lock().unlock();

    pid
}

pub fn update_running_process_to_ready() {
    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if process.state == ProcessState::Running(cpu::get_mhartid()) {
            process.state = ProcessState::Ready;
        }
    }

    get_process_list_lock().unlock();
}

pub fn update_running_process_to_blocked() {
    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if process.state == ProcessState::Running(cpu::get_mhartid()) {
            process.state = ProcessState::Blocked;
        }
    }

    get_process_list_lock().unlock();
}

pub fn update_running_process_to_sleeping(until: usize) {
    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if process.state == ProcessState::Running(cpu::get_mhartid()) {
            process.state = ProcessState::Sleeping(until);
        }
    }

    get_process_list_lock().unlock();
}

pub fn get_running_process_blocking_pid() -> Option<usize> {
    let mut blocking_pid = None;
    get_process_list_lock().spin_lock();

    for process in unsafe { PROCESS_LIST.as_mut().unwrap() } {
        if process.state == ProcessState::Running(cpu::get_mhartid()) {
            blocking_pid = process.blocking_pid;
        }
    }

    get_process_list_lock().unlock();
    blocking_pid
}

pub fn delete_running_process() {
    get_process_list_lock().spin_lock();

    if let Some(mut process_list) = unsafe { PROCESS_LIST.take() } {
        process_list.remove(
            process_list
                .iter()
                .enumerate()
                .find(|(_i, proc)| proc.state == ProcessState::Running(cpu::get_mhartid()))
                .unwrap()
                .0,
        );
        unsafe {
            PROCESS_LIST.replace(process_list);
        }
    }

    get_process_list_lock().unlock();
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

pub fn switch_to_process(trap_frame: *const TrapFrame) -> ! {
    unsafe { assembly::__tong_os_switch_to_process(trap_frame) }
}

pub fn idle() {
    loop {
        unsafe { asm!("wfi") }
    }
}
