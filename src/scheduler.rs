// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::process::{self, Process, ProcessState};

// next_process = schedule()
// switch_context(current_process, next_process)
pub fn schedule() -> &'static Option<Process> {
    // Try to get process list reference
    unsafe {
        println!("process_list = {:?}", process::PROCESS_LIST);
        if let Some(mut process_list) = process::PROCESS_LIST.take() {
            // Shift list to left
            println!("process_list = {:?}", process_list);

            process_list.rotate_left(1);
            // Get first process
            if let Some(first) = process_list.pop_front() {
                match first.state {
                    ProcessState::Ready => {
                        process::RRUNNING.replace(first);
                        return &process::RRUNNING;
                    }
                    ProcessState::Running => {}
                    ProcessState::Blocked => {}
                }
            }
        }
    }

    &None
}
