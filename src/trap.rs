// trap.rs
// Trap routines
// Stephen Marz
// tongOS team

use crate::cpu::CONTEXT_SWITCH_TIME;
use crate::process::{self, Process};
use crate::scheduler::schedule;

pub fn init() {
    use crate::assembly::__tong_os_trap;

    let mstatus: usize;
    unsafe { asm!("csrr {}, mstatus", out(reg) mstatus) }

    // [3] = MIE (Machine Interrupt Enable)
    let flags = 1 << 3;
    let mstatus = mstatus | flags;
    unsafe { asm!("csrw mstatus, {}", in(reg) mstatus) }

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
        MMIO_MTIMECMP.write_volatile(
            MMIO_MTIME
                .read_volatile()
                .wrapping_add(CONTEXT_SWITCH_TIME * quantum as u64),
        );
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
            7 => {
                println!("Handling async interrupt {} to process#{}", cause, process.pid);
                unsafe {
                    let mut old_process = process::PROCESS_RUNNING.take().unwrap();
                    old_process.state = process::ProcessState::Ready;
                    process::process_list_add(old_process);
                    if let Some(next_process) = schedule() {
                        println!("interrupt process {}, pc={:x}", next_process.pid, next_process.context.pc);
                        schedule_machine_timer_interrupt(next_process.quantum);
                        process::switch_to_user(&next_process);
                    } else {
                        panic!("Next process not found!");
                    }
                }
                panic!("Unhandled machine timer interrupt!!");
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
            8 | 9 | 11 => {
                println!("Handling sync exception {} to process#{}", cause, process.pid);
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
                        schedule_machine_timer_interrupt(next_process.quantum);
                        process::switch_to_user(&next_process);
                    } else {
                        panic!("Next process not found!");
                    }
                }

                // Check if process has a child to schedule
                if let Some(child_proc) = process.schedule_child() {
                    schedule_machine_timer_interrupt(child_proc.quantum);
                    process::switch_to_user(&child_proc);
                } else {
                    // Otherwise, normal scheduling
                    if let Some(next_process) = schedule() {
                        println!("interrupt process {}, pc={:x}", next_process.pid, next_process.context.pc);
                        schedule_machine_timer_interrupt(next_process.quantum);
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
