// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::process::{self, Process, ProcessState};
use crate::cpu;

pub fn schedule() -> &'static Option<Process> {
    // Try to get process list reference
    debug!("SCHEDULING!");
    // process::print_process_list();
    unsafe {
        process::get_process_list_lock().spin_lock();
        if let Some(mut process_list) = process::PROCESS_LIST.take() {
            loop {
                // Get first process
                if let Some(p) = process_list.front() {
                    match p.state {
                        ProcessState::Ready => {
                            let mut first = process_list.pop_front().unwrap();
                            first.state = ProcessState::Running;
                            process::PROCESS_RUNNING[cpu::get_hartid()].replace(first);

                            process::PROCESS_LIST.replace(process_list);
                            process::get_process_list_lock().unlock();
                            return &process::PROCESS_RUNNING[cpu::get_hartid()];
                        }
                        ProcessState::Running => {}
                        ProcessState::Sleeping => {
                            let current_time = crate::trap::MMIO_MTIME.read_volatile() as usize;

                            if p.sleep_until < current_time {
                                let mut first = process_list.pop_front().unwrap();
                                first.state = ProcessState::Running;
                                process::PROCESS_RUNNING[cpu::get_hartid()].replace(first);

                                process::PROCESS_LIST.replace(process_list);
                                process::get_process_list_lock().unlock();
                                return &process::PROCESS_RUNNING[cpu::get_hartid()];
                            } else {
                                process_list.rotate_left(1);
                            }
                        }
                        ProcessState::Blocked => {
                            process_list.rotate_left(1);
                        }
                    }
                } else {
                    process::get_process_list_lock().unlock();
                    panic!("No more processes for hart {}! Shutting down...", cpu::get_hartid());
                }
            }
        }
    }

    &None
}
