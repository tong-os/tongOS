// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::process::{self, Process, ProcessState};

// next_process = schedule()
// switch_context(current_process, next_process)
pub fn schedule() -> &'static Option<Process> {
    // Try to get process list reference
    debug!("SCHEDULING!");
    // process::print_process_list();
    unsafe {
        if let Some(mut process_list) = process::PROCESS_LIST.take() {
            loop {
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
                        ProcessState::Sleeping => {
                            let current_time = crate::trap::MMIO_MTIME.read_volatile() as usize;

                            if p.sleep_until < current_time {
                                let mut first = process_list.pop_front().unwrap();
                                first.state = ProcessState::Running;
                                process::PROCESS_RUNNING.replace(first);

                                process::PROCESS_LIST.replace(process_list);
                                return &process::PROCESS_RUNNING;
                            } else {
                                process_list.rotate_left(1);
                            }
                        }
                        ProcessState::Blocked => {
                            process_list.rotate_left(1);
                        }
                    }
                } else {
                    panic!("No more processes! Shutting down...")
                }
            }
        }
    }

    &None
}
