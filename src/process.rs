// process.rs
// Kernel and user processes
// Stephen Marz
// tongOS team

use crate::assembly;
use crate::cpu::{self, CpuMode, TrapFrame};
use crate::lock::Mutex;
use crate::page::{self, PageTableEntryFlags, Sv39PageTable};

use alloc::collections::vec_deque::VecDeque;

pub const IDLE_ID: usize = core::usize::MAX;

static mut PROCESS_READY: [Option<VecDeque<Process>>; 4] = [None, None, None, None];
static mut PROCESS_BLOCKED: [Option<VecDeque<Process>>; 4] = [None, None, None, None];
static mut PROCESS_SLEEPING: [Option<VecDeque<Process>>; 4] = [None, None, None, None];
static mut PROCESS_RUNNING: [Option<Process>; 4] = [None, None, None, None];
static mut PROCESS_IDLE: [Option<Process>; 4] = [None, None, None, None];
static mut PROCESS_LIST_LOCK: [Mutex; 4] = [Mutex::new(); 4];

pub fn running_process() -> &'static Process {
    unsafe { PROCESS_RUNNING[cpu::get_mhartid()].as_ref().unwrap() }
}

fn running_process_mut() -> &'static mut Process {
    unsafe { PROCESS_RUNNING[cpu::get_mhartid()].as_mut().unwrap() }
}

fn running_process_take() -> Process {
    unsafe { PROCESS_RUNNING[cpu::get_mhartid()].take().unwrap() }
}

pub fn running_process_replace(running: Process) {
    unsafe { PROCESS_RUNNING[cpu::get_mhartid()].replace(running) };
}

pub fn idle_process_take() -> Process {
    unsafe { PROCESS_IDLE[cpu::get_mhartid()].take().unwrap() }
}

pub fn idle_process_replace(running: Process) {
    unsafe { PROCESS_IDLE[cpu::get_mhartid()].replace(running) };
}

pub fn running_list() -> &'static [Option<Process>] {
    unsafe { PROCESS_RUNNING.as_ref() }
}

fn running_list_mut() -> &'static mut [Option<Process>] {
    unsafe { PROCESS_RUNNING.as_mut() }
}

fn ready_list() -> &'static VecDeque<Process> {
    unsafe { PROCESS_READY[cpu::get_mhartid()].as_ref().unwrap() }
}

pub fn ready_list_mut() -> &'static mut VecDeque<Process> {
    unsafe { PROCESS_READY[cpu::get_mhartid()].as_mut().unwrap() }
}

fn blocked_list() -> &'static VecDeque<Process> {
    unsafe { PROCESS_BLOCKED[cpu::get_mhartid()].as_ref().unwrap() }
}

fn blocked_list_mut() -> &'static mut VecDeque<Process> {
    unsafe { PROCESS_BLOCKED[cpu::get_mhartid()].as_mut().unwrap() }
}

fn sleeping_list() -> &'static VecDeque<Process> {
    unsafe { PROCESS_SLEEPING[cpu::get_mhartid()].as_ref().unwrap() }
}

fn sleeping_list_mut() -> &'static mut VecDeque<Process> {
    unsafe { PROCESS_SLEEPING[cpu::get_mhartid()].as_mut().unwrap() }
}

pub fn get_process_list_lock() -> &'static mut Mutex {
    unsafe { &mut PROCESS_LIST_LOCK[cpu::get_mhartid()] }
}

static DEFAULT_QUANTUM: usize = 1;

static mut NEXT_PID: usize = 0;
static mut NEXT_PID_LOCK: Mutex = Mutex::new();

pub fn get_next_pid() -> usize {
    unsafe {
        NEXT_PID_LOCK.spin_lock();
        let pid = {
            NEXT_PID += 1;
            NEXT_PID
        };
        NEXT_PID_LOCK.unlock();
        pid
    }
}

pub fn init() {
    unsafe {
        for list in PROCESS_READY.as_mut().iter_mut() {
            list.replace(VecDeque::new());
        }
    }
    unsafe {
        for list in PROCESS_SLEEPING.as_mut().iter_mut() {
            list.replace(VecDeque::new());
        }
    }
    unsafe {
        for list in PROCESS_BLOCKED.as_mut().iter_mut() {
            list.replace(VecDeque::new());
        }
    }
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
    pub fn new(start: usize, arg0: usize, arg1: usize, arg2: usize) -> Self {
        let pid = get_next_pid();

        let page_table_address = page::zalloc(1);
        assert!(page_table_address as *const u8 != core::ptr::null());

        let mut context = TrapFrame::new();
        context.regs[cpu::GeneralPurposeRegister::A0 as usize] = arg0;
        context.regs[cpu::GeneralPurposeRegister::A1 as usize] = arg1;
        context.regs[cpu::GeneralPurposeRegister::A2 as usize] = arg2;
        context.satp = cpu::build_satp(pid, page_table_address as usize);
        context.pc = start as usize;
        context.global_interrupt_enable = 0;
        context.mode = CpuMode::User as usize;

        let num_stack_pages = 12;
        let stack = page::zalloc(num_stack_pages) as usize;
        assert!(stack as *const u8 != core::ptr::null());
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
            trap_frame,
            stack: stack as *mut u8,
            state: ProcessState::Ready,
            page_table,
            quantum: DEFAULT_QUANTUM,
            pid,
            blocking_pid: None,
            sleep_until: 0,
        }
    }

    pub fn new_idle() -> Self {
        let mut context = TrapFrame::new();
        context.pc = self::idle as usize;
        context.global_interrupt_enable = 1;
        context.mode = CpuMode::Machine as usize;

        let num_stack_pages = 2;
        let stack = page::zalloc(num_stack_pages) as usize;
        assert!(stack as *const u8 != core::ptr::null());
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
            pid: IDLE_ID,
            blocking_pid: None,
            sleep_until: 0,
        };

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

fn make_user_syscall(_arg0: usize, _arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize) {
    unsafe {
        asm!("ECALL");
    }
}

pub fn create_thread(func: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    make_user_syscall(1, func, arg0, arg1, arg2);
    let pid: usize;
    unsafe {
        asm!("mv {}, a0", out(reg) pid);
    }
    pid
}

pub fn exit() {
    make_user_syscall(0, 0, 0, 0, 0);
}

pub fn join(pid: usize) {
    make_user_syscall(2, pid, 0, 0, 0);
}

pub fn sleep(amount: usize) {
    make_user_syscall(3, amount, 0, 0, 0);
}

pub fn read_line(buffer: &mut alloc::string::String) {
    make_user_syscall(4, buffer as *mut _ as usize, 0, 0, 0);
    while unsafe { crate::uart::READING } {}
}

pub fn print_str(buffer: &str) {
    make_user_syscall(5, buffer.as_ptr() as usize, buffer.len(), 0, 0);
}

pub fn time_now() -> usize {
    make_user_syscall(6, 0, 0, 0, 0);
    let time: usize;
    unsafe {
        asm!("mv {}, a0", out(reg) time);
    }
    time
}

pub fn set_blocking_pid(pid: usize, blocking_pid: usize) {
    get_process_list_lock().spin_lock();

    if let Some(process) = running_list_mut().iter_mut().find(|op| {
        if let Some(process) = op {
            process.pid == pid
        } else {
            false
        }
    }) {
        process.as_mut().unwrap().blocking_pid = Some(blocking_pid);
    }

    if let Some(process) = sleeping_list_mut().iter_mut().find(|p| p.pid == pid) {
        process.blocking_pid = Some(blocking_pid);
    }

    if let Some(process) = blocked_list_mut().iter_mut().find(|p| p.pid == pid) {
        process.blocking_pid = Some(blocking_pid);
    }

    if let Some(process) = ready_list_mut().iter_mut().find(|p| p.pid == pid) {
        process.blocking_pid = Some(blocking_pid);
    }

    get_process_list_lock().unlock();
}

pub fn try_wake_sleeping() -> bool {
    let mut woken = false;
    get_process_list_lock().spin_lock();

    let mut iter = sleeping_list().iter();
    let current_time = crate::trap::get_mtime() as usize;
    while let Some(pos) = iter.position(|p| {
        if let ProcessState::Sleeping(until) = p.state {
            if until < current_time {
                true
            } else {
                false
            }
        } else {
            false
        }
    }) {
        woken = true;
        let mut woken = sleeping_list_mut().swap_remove_back(pos).unwrap();
        woken.state = ProcessState::Ready;
        debug!("woken pid {}", woken.pid);
        ready_list_mut().push_back(woken);
    }

    get_process_list_lock().unlock();
    woken
}

pub fn unblock_process_by_pid(blocked_pid: usize) {
    get_process_list_lock().spin_lock();

    if let Some(pos) = blocked_list().iter().position(|p| p.pid == blocked_pid) {
        let mut woken = blocked_list_mut().remove(pos).unwrap();
        woken.state = ProcessState::Ready;
        ready_list_mut().push_back(woken);
    }

    get_process_list_lock().unlock();
}

pub fn process_list_add(process: Process) {
    get_process_list_lock().spin_lock();
    debug!("process list add pid {}", process.pid);

    ready_list_mut().push_back(process);

    get_process_list_lock().unlock();
}

pub fn process_list_contains(pid: usize) -> bool {
    get_process_list_lock().spin_lock();

    if let Some(_proc) = running_list().iter().find(|op| {
        if let Some(p) = op {
            p.pid == pid
        } else {
            false
        }
    }) {
        get_process_list_lock().unlock();
        return true;
    }

    if let Some(_proc) = blocked_list().iter().find(|p| p.pid == pid) {
        get_process_list_lock().unlock();
        return true;
    }

    if let Some(_proc) = sleeping_list().iter().find(|p| p.pid == pid) {
        get_process_list_lock().unlock();
        return true;
    }

    if let Some(_proc) = ready_list().iter().find(|p| p.pid == pid) {
        get_process_list_lock().unlock();
        return true;
    }

    get_process_list_lock().unlock();
    false
}

pub fn update_running_process_trap_frame(trap_frame: *mut TrapFrame) {
    get_process_list_lock().spin_lock();
    running_process_mut().trap_frame = trap_frame;
    get_process_list_lock().unlock();
}

pub fn get_running_process_pid() -> usize {
    get_process_list_lock().spin_lock();

    let pid = running_process().pid;

    get_process_list_lock().unlock();

    pid
}

// takes the running process and put in the ready list
pub fn yield_running_process() {
    get_process_list_lock().spin_lock();

    let mut running = running_process_take();
    running.state = ProcessState::Ready;

    ready_list_mut().push_back(running);

    get_process_list_lock().unlock();
}

pub fn block_process() {
    get_process_list_lock().spin_lock();

    let mut running = running_process_take();
    running.state = ProcessState::Blocked;

    blocked_list_mut().push_back(running);

    get_process_list_lock().unlock();
}

pub fn put_process_to_sleep(until: usize) {
    get_process_list_lock().spin_lock();

    let mut running = running_process_take();

    running.state = ProcessState::Sleeping(until);

    sleeping_list_mut().push_back(running);

    get_process_list_lock().unlock();
}

// takes the running idle and give back to idle list
pub fn yield_idle_process() {
    get_process_list_lock().spin_lock();

    let mut running = running_process_take();
    running.state = ProcessState::Ready;

    idle_process_replace(running);

    get_process_list_lock().unlock();
}

pub fn get_running_process_blocking_pid() -> Option<usize> {
    get_process_list_lock().spin_lock();

    let blocking_pid = running_process().blocking_pid;

    get_process_list_lock().unlock();
    blocking_pid
}

pub fn delete_running_process() {
    get_process_list_lock().spin_lock();

    drop(running_process_take());

    get_process_list_lock().unlock();
}

pub fn print_process_list() {
    debug!("------ running:");
    for proc in running_list() {
        if let Some(proc) = proc {
            debug!("pid: {} {:?}", proc.pid, proc.state);
        } else {
            debug!("None");
        }
    }
    debug!("------ ready:");
    for proc in ready_list() {
        debug!("pid: {} {:?}", proc.pid, proc.state);
    }
    debug!("------ blocked:");
    for proc in blocked_list() {
        debug!("pid: {} {:?}", proc.pid, proc.state);
    }
    debug!("------ sleeping:");
    for proc in sleeping_list() {
        debug!("pid: {} {:?}", proc.pid, proc.state);
    }
    debug!("------ idle:");
    for proc in unsafe { PROCESS_IDLE.as_ref() } {
        if let Some(proc) = proc {
            debug!("pid: {} {:?}", proc.pid, proc.state);
        } else {
            debug!("None");
        }
    }
    debug!("-----------");
}

pub fn switch_to_process(trap_frame: *const TrapFrame) -> ! {
    unsafe { assembly::__tong_os_switch_to_process(trap_frame) }
}

pub fn idle() -> ! {
    loop {
        unsafe { asm!("wfi") }
    }
}
