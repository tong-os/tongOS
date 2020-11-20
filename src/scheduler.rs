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
        let mut all_none = false;
        if process::PROCESS_LIST.as_ref().unwrap().is_empty() {
            process::get_pid_list_lock().spin_lock();
            if process::PID_LIST.as_ref().unwrap().is_empty() {
                all_none = true;
            }
            process::get_pid_list_lock().unlock();
            if all_none {
                process::get_process_list_lock().unlock();
                // tong_os::get_print_lock().spin_lock();
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
                            let mut last_blocked_pid = false;

                            if process_list.len() == 1 {
                                process::get_pid_list_lock().spin_lock();
                                if let Some(pid_list) = process::PID_LIST.take() {
                                    if pid_list.len() == 1 {
                                        last_blocked_pid = true;
                                    }

                                    process::PID_LIST.replace(pid_list);
                                }
                                process::get_pid_list_lock().unlock();

                                // force process to wake
                                if last_blocked_pid {
                                    let mut first = process_list.pop_front().unwrap();
                                    first.state = ProcessState::Running;
                                    process::PROCESS_RUNNING[cpu::get_hartid()].replace(first);

                                    process::PROCESS_LIST.replace(process_list);
                                    process::get_process_list_lock().unlock();

                                    return &process::PROCESS_RUNNING[cpu::get_hartid()];
                                } else {
                                    process::PROCESS_LIST.replace(process_list);
                                    process::get_process_list_lock().unlock();

                                    return &process::PROCESS_IDLE[cpu::get_hartid()];
                                }
                            }

                            process_list.rotate_left(1);
                        }
                    }
                } else {
                    process::PROCESS_LIST.replace(process_list);
                    process::get_process_list_lock().unlock();

                    process::PROCESS_RUNNING[cpu::get_hartid()].take();
                    return &process::PROCESS_IDLE[cpu::get_hartid()];
                }
            }
        }
    }

    &None
}
