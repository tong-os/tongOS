// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::cpu;
use crate::process::{self, ProcessState};
use crate::trap;

pub fn schedule() -> ! {
    process::get_process_list_lock().spin_lock();
    debug!("running schedule");

    // process::print_process_list();

    if let Some(mut next) = process::ready_list_mut().pop_front() {
        next.state = ProcessState::Running(cpu::get_mhartid());
        debug!("scheduling pid {}", next.pid);
        process::running_process_replace(next);
        let trap_frame = process::running_process().trap_frame;
        let quantum = process::running_process().quantum;
        process::get_process_list_lock().unlock();
        trap::schedule_machine_timer_interrupt(quantum);
        trap::disable_machine_software_interrupt();
        process::switch_to_process(trap_frame);
    } else {
        let mut idle = process::idle_process_take();
        idle.state = ProcessState::Running(cpu::get_mhartid());
        debug!("scheduling idle");
        process::running_process_replace(idle);
        let trap_frame = process::running_process().trap_frame;
        let quantum = process::running_process().quantum;
        process::get_process_list_lock().unlock();
        trap::schedule_machine_timer_interrupt(quantum);
        trap::complete_software_interrupt(cpu::get_mhartid());
        trap::enable_machine_software_interrupt();
        process::switch_to_process(trap_frame);
    }
}
