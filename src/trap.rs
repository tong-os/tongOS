use crate::process::{self, Process};
use crate::scheduler::schedule;
use alloc::boxed::Box;

pub fn init() {
    use crate::assembly::__tong_os_trap;
    unsafe {
        asm!("csrw mtvec, {}", in(reg) (__tong_os_trap as usize));
    }
}

#[no_mangle]
pub fn tong_os_trap(process: &mut Process) {
    let mcause: usize;
    unsafe {
        asm!("csrr {}, mcause", out(reg) mcause);
    }
    println!("In tongo_os_trap!");

    // Get interrupt bit from mcause
    let is_async = mcause >> 63 & 1 == 1;
    // Get interrupt cause
    let cause = mcause & 0xfff;

    if is_async {
        match cause {
            _ => {
                panic!(
                    "Unhandled async trap CPU#{} -> {}\n",
                    process.context.hartid, cause
                );
            }
        }
    } else {
        match cause {
            8 | 9 | 11 => {
                println!("Handling exception {} to process#{}", cause, process.pid);
                // Check if child process needs to reschedule parent
                if let Some(_ppid) = process.ppid {
                    // Otherwise, normal scheduling
                    println!("Scheduling father process");
                    process::print_process_list();

                    if let Some(next_process) = schedule() {
                        println!("PPID={}", next_process.pid);
                        unsafe {
                            process::PROCESS_RUNNING.as_mut().unwrap().context.pc += 4;
                        }
                        process::switch_to_user(&next_process);
                    } else {
                        panic!("Next process not found!");
                    }
                }

                // Check if process has a child to schedule
                if let Some(child_proc) = process.schedule_child() {
                    process::switch_to_user(&child_proc);
                } else {
                    // Otherwise, normal scheduling
                    if let Some(next_process) = schedule() {
                        process::switch_to_user(&next_process);
                    } else {
                        panic!("Next process not found!");
                    }
                }
            }
            cause => {
                let mtval: usize;
                unsafe {
                    asm!("csrr {}, mtval", out(reg) mtval);
                }
                panic!(
                    "Unhandled sync trap CPU#{} -> cause: {}; mval: {:x?}\n",
                    process.context.hartid, cause, mtval
                );
            }
        }
    }
}
