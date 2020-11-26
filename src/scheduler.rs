// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::cpu::{self, TrapFrame};
use crate::process::{self, Process, ProcessState};
use crate::trap;

pub fn schedule() -> ! {
    process::get_process_list_lock().spin_lock();
    debug!("running schedule");

    if let Some(next) = process::ready_list_mut().pop_front() {
        debug!("scheduling pid {}", next.pid);
        let (trap_frame, quantum) = prepare_running_process(next);

        process::get_process_list_lock().unlock();

        trap::disable_machine_software_interrupt();
        trap::schedule_machine_timer_interrupt(quantum);
        process::switch_to_process(trap_frame);
    } else {
        debug!("scheduling idle");
        let idle = process::idle_process_take();
        let (trap_frame, quantum) = prepare_running_process(idle);

        process::get_process_list_lock().unlock();

        trap::complete_software_interrupt(cpu::get_mhartid());
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