// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::cpu;
use crate::process::{self, Process, ProcessState};

pub fn schedule() -> &'static Option<Process> {
    // Try to get process list reference
    debug!("SCHEDULING!");
    // process::print_process_list();
    unsafe {
        process::get_process_list_lock().spin_lock();
        if process::PROCESS_LIST.as_ref().unwrap().is_empty() {
            println!("empty process list");
            let mut all_none = true;
            for process in process::PROCESS_RUNNING.as_ref().iter() {
                if let Some(proc) = process {
                    println!("pid : {}", proc.pid);
                    all_none = false;
                    break;
                }
            }
            if all_none {
                process::get_process_list_lock().unlock();
                panic!("Shutting down hart {}, no more process.", cpu::get_hartid());
            }
        }
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
                    process::PROCESS_LIST.replace(process_list);
                    process::get_process_list_lock().unlock();
                    // println!("schedule idel on hart {}", cpu::get_hartid());
                    return &process::PROCESS_IDLE[cpu::get_hartid()];
                }
            }
        }
    }

    &None
}
