// trap.rs
// Trap routines
// Stephen Marz
// tongOS team

use crate::cpu::{GeneralPurposeRegister, CONTEXT_SWITCH_TIME};
use crate::process::{self, Process};
use crate::scheduler::schedule;

pub fn init() {
    use crate::assembly::__tong_os_trap;

    // configure mstatus
    // enable_global_interrupts();

    let mie: usize;
    unsafe { asm!("csrr {}, mie", out(reg) mie) }

    // [7] = MTIE (Machine Time Interrupt Enable)
    let flags = 1 << 7;
    let mie = mie | flags;
    unsafe { asm!("csrw mie, {}", in(reg) mie) }

    unsafe { asm!("csrw mtvec, {}", in(reg) (__tong_os_trap as usize)) }
}

// CLINT Memory Map
// https://sifive.cdn.prismic.io/sifive/b5e7a29c-d3c2-44ea-85fb-acc1df282e21_FU540-C000-v1p3.pdf
pub const MMIO_MTIMECMP: *mut u64 = 0x0200_4000usize as *mut u64;
pub const MMIO_MTIME: *const u64 = 0x0200_BFF8 as *const u64;

pub fn schedule_machine_timer_interrupt(quantum: usize) {
    unsafe {
        if crate::ENABLE_PREEMPTION {
            MMIO_MTIMECMP.write_volatile(
                MMIO_MTIME
                    .read_volatile()
                    .wrapping_add(CONTEXT_SWITCH_TIME * quantum as u64),
            );
        }
    }
}

#[no_mangle]
pub fn tong_os_trap(process: &mut Process) {
    let mcause: usize;
    unsafe {
        asm!("csrr {}, mcause", out(reg) mcause);
    }
    debug!("In tongo_os_trap!");

    // Get interrupt bit from mcause
    let is_async = mcause >> 63 & 1 == 1;
    // Get interrupt cause
    let cause = mcause & 0xfff;

    if is_async {
        match cause {
            7 => {
                debug!(
                    "Handling async timer interrupt: mcause {}, pid {}",
                    cause, process.pid
                );
                unsafe {
                    let mut old_process = process::PROCESS_RUNNING.take().unwrap();
                    old_process.state = process::ProcessState::Ready;
                    process::process_list_add(old_process);
                    if let Some(next_process) = schedule() {
                        debug!(
                            "interrupt process {}, pc={:x}",
                            next_process.pid, next_process.context.pc
                        );
                        schedule_machine_timer_interrupt(next_process.quantum);
                        process::switch_to_user(&next_process);
                    } else {
                        panic!("Next process not found!");
                    }
                }
            }
            _ => {
                panic!(
                    "Unhandled async trap CPU#{} -> {}\n",
                    process.context.hartid, cause
                );
            }
        }
    } else {
        match cause {
            8 => {
                let which_code = process.context.regs[GeneralPurposeRegister::A0 as usize];

                debug!(
                    "Handling user ecall exception: mcause {}, pid {}, syscall code {}",
                    cause, process.pid, which_code,
                );

                match which_code {
                    // Exiting process
                    0 => {
                        // Check if child process needs to reschedule parent
                        if let Some(blocked_pid) = process.blocking_pid {
                            process::wake_process(blocked_pid);
                        }

                        if let Some(next_process) = schedule() {
                            debug!(
                                "interrupt process {}, pc={:x}",
                                next_process.pid, next_process.context.pc
                            );
                            schedule_machine_timer_interrupt(next_process.quantum);
                            process::switch_to_user(&next_process);
                        } else {
                            panic!("Next process not found!");
                        }
                    }
                    // Create thread
                    1 => {
                        let process_address =
                            process.context.regs[GeneralPurposeRegister::A1 as usize];
                        let process_arg = process.context.regs[GeneralPurposeRegister::A2 as usize];
                        let new_process = process::Process::new(process_address, process_arg);
                        let new_process_pid = new_process.pid;
                        process::process_list_add(new_process);
                        process.context.regs[GeneralPurposeRegister::A0 as usize] = new_process_pid;
                        process.context.pc += 4;
                        process::switch_to_user(process);
                    }
                    2 => {
                        let joining_pid = process.context.regs[GeneralPurposeRegister::A1 as usize];

                        // if joining pid has already exited
                        if !process::process_list_contains(joining_pid) {
                            // add runnign to proc list as readdy and schedule
                            let mut running = unsafe { process::PROCESS_RUNNING.take().unwrap() };
                            running.state = process::ProcessState::Ready;
                            running.context.pc += 4;
                            process::process_list_add(running);
                            if let Some(next) = schedule() {
                                schedule_machine_timer_interrupt(next.quantum);
                                process::switch_to_user(next);
                            } else {
                                panic!("Joining non existent process failure");
                            }
                        } else {
                            let mut running = unsafe { process::PROCESS_RUNNING.take().unwrap() };
                            running.state = process::ProcessState::Blocked;
                            running.context.pc += 4;
                            let blocking_pid = running.pid;
                            process::process_list_add(running);
                            process::set_blocking_pid(joining_pid, blocking_pid);
                            if let Some(next) = schedule() {
                                schedule_machine_timer_interrupt(next.quantum);
                                process::switch_to_user(next);
                            } else {
                                panic!("Joining existent process failure");
                            }
                        }
                    }
                    code => {
                        panic!("Unhandled user ecall with code {}", code);
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
