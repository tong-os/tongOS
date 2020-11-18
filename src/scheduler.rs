// sched.rs
// Simple process scheduler
// Stephen Marz
// tongOs team

use crate::cpu;
use crate::process::{self, ProcessState};
use crate::trap;

pub fn schedule() -> ! {
    unsafe {
        process::get_process_list_lock().spin_lock();
        debug!("running schedule");
        // if process::PROCESS_LIST.as_ref().unwrap().is_empty() {
        //     process::get_process_list_lock().unlock();
        //     panic!(
        //         "shutting down hart {}, no more process.",
        //         cpu::get_mhartid()
        //     );
        // }
        let mut blocked_count = 0;
        if let Some(mut process_list) = process::PROCESS_LIST.take() {
            for proc in process_list.iter() {
                debug!("proc list pid {} state {:?}", proc.pid, proc.state);
            }
            loop {
                // Get first process
                if let Some(front) = process_list.front_mut() {
                    match front.state {
                        ProcessState::Ready => {
                            front.state = ProcessState::Running(cpu::get_mhartid());
                            let trap_frame = front.trap_frame;
                            let quantum = front.quantum;
                            debug!("scheduling pid {}", front.pid);
                            process_list.rotate_left(1);
                            process::PROCESS_LIST.replace(process_list);
                            process::get_process_list_lock().unlock();
                            trap::schedule_machine_timer_interrupt(quantum);
                            process::switch_to_process(trap_frame);
                        }
                        ProcessState::Running(_) => {
                            process_list.rotate_left(1);
                            blocked_count += 1;
                            if blocked_count == process_list.len() {
                                debug!("break on running");
                                break;
                            }
                        }
                        ProcessState::Sleeping(until) => {
                            blocked_count = 0;

                            let current_time = trap::get_mtime() as usize;

                            if until < current_time {
                                front.state = ProcessState::Running(cpu::get_mhartid());
                                let trap_frame = front.trap_frame;
                                let quantum = front.quantum;
                                debug!("scheduling pid {}", front.pid);
                                process_list.rotate_left(1);
                                process::PROCESS_LIST.replace(process_list);
                                process::get_process_list_lock().unlock();
                                trap::schedule_machine_timer_interrupt(quantum);
                                process::switch_to_process(trap_frame);
                            } else {
                                process_list.rotate_left(1);
                            }
                        }
                        ProcessState::Blocked => {
                            process_list.rotate_left(1);
                            blocked_count += 1;
                            if blocked_count == process_list.len() {
                                debug!("break on blocked");
                                break;
                            }
                        }
                    }
                } else {
                    break;
                }
            }
            process::PROCESS_LIST.replace(process_list);
            debug!("scheduling idle");
            process::get_process_list_lock().unlock();
            trap::disable_machine_timer_interrupt();
            let idle = process::PROCESS_IDLE[cpu::get_mhartid()].as_mut().unwrap();
            idle.state = ProcessState::Running(cpu::get_mhartid());
            process::switch_to_process(
                process::PROCESS_IDLE[cpu::get_mhartid()]
                    .as_ref()
                    .unwrap()
                    .trap_frame,
            );
        }
    }
    panic!("Scheduler could not schedule!");
}
