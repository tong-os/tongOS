// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::process::{self, print_process_list, Process, ProcessState};

// next_process = schedule()
// switch_context(current_process, next_process)
pub fn schedule() -> &'static Option<Process> {
    // Try to get process list reference
    println!("SCHEDULING!");
    print_process_list();
    unsafe {
        if let Some(mut process_list) = process::PROCESS_LIST.take() {
            loop {
                // Shift list to left
                process_list.rotate_left(1);
                // Get first process
                if let Some(p) = process_list.front() {
                    match p.state {
                        ProcessState::Ready => {
                            let mut first = process_list.pop_front().unwrap();
                            first.state = ProcessState::Running;
                            process::PROCESS_RUNNING.replace(first);

                            process::PROCESS_LIST.replace(process_list);
                            return &process::PROCESS_RUNNING;
                        }
                        ProcessState::Running => {}
                        ProcessState::Blocked => {}
                    }
                }
            }
        }
    }

    &None
}
