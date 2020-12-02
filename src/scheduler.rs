// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::cpu::{self, TrapFrame};
use crate::lock::Mutex;
use crate::process::{self, Process, ProcessState};
use crate::trap;

const CRITERIA: usize = 1; 

fn next_hart_criteria() -> usize {
    (cpu::get_mhartid() + 1) % 4
}

static mut NEXT_HART: usize = 0;
static mut NEXT_HART_MUTEX: Mutex = Mutex::new();

fn round_robin_criteria() -> usize {
    unsafe {
        NEXT_HART_MUTEX.spin_lock();

        let next_hart = {
            let next_hart = NEXT_HART;
            NEXT_HART = (NEXT_HART + 1) % 4;
            next_hart
        };

        NEXT_HART_MUTEX.unlock();
        next_hart
    }
}

fn least_busy() -> usize {
    let mut least = core::usize::MAX;
    let mut least_hartid = 0;
    for hartid in 0..4 {
        if let Some(process) = process::running_list()[hartid].as_ref() {
            if process.pid == process::IDLE_ID {
                return hartid;
            }
        };
        let len = process::ready_list_by_hartid_mut(hartid).len();
        if  len < least {
            least = len;
            least_hartid = hartid;
        }
    }
    least_hartid
}

pub fn migration_criteria() -> usize {
    match CRITERIA {
        0 => least_busy(),
        1 => round_robin_criteria(),
        2 => next_hart_criteria(),
        _ => panic!("invalid migration criteri"),
    }
}

pub fn schedule() -> ! {
    process::get_ready_list_lock().spin_lock();
    debug!("running schedule");

    if let Some(next) = process::ready_list_mut().pop_front() {
        debug!("scheduling pid {}", next.pid);
        let (trap_frame, quantum) = prepare_running_process(next);

        process::get_ready_list_lock().unlock();

        trap::disable_machine_software_interrupt();
        trap::schedule_machine_timer_interrupt(quantum);
        process::switch_to_process(trap_frame);
    } else {
        debug!("scheduling idle");
        let idle = process::idle_process_take();
        let (trap_frame, quantum) = prepare_running_process(idle);

        process::get_ready_list_lock().unlock();

        trap::enable_machine_software_interrupt();
        trap::schedule_machine_timer_interrupt(quantum);
        process::switch_to_process(trap_frame);
    }
}

fn prepare_running_process(mut next: Process) -> (*const TrapFrame, usize) {
    next.state = ProcessState::Running(cpu::get_mhartid());
    let trap_frame = next.trap_frame;
    let quantum = next.quantum;
    process::running_process_replace(next);
    (trap_frame, quantum)
}
