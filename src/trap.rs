// trap.rs
// Trap routines
// Stephen Marz
// tongOS team

use crate::cpu::{self, GeneralPurposeRegister, TrapFrame};
use crate::plic;
use crate::process;
use crate::scheduler;
use crate::uart;

pub fn init() {
    use crate::assembly::__tong_os_trap_machine_mode;

    unsafe { asm!("csrw mtvec, {}", in(reg) (__tong_os_trap_machine_mode as usize)) }

    // configure mstatus
    // enable_global_interrupts();

    // [7] = MTIE (Machine Time Interrupt Enable)
    let flags = 1 << 7 | 1 << 3;
    unsafe { asm!("csrw mie, {}", in(reg) flags) }
}

pub fn disable_machine_timer_interrupt() {
    let flags: usize;
    unsafe { asm!("csrr {}, mie", out(reg) flags) }
    let flags_mask = !(1 << 7);
    unsafe { asm!("csrw mie, {}", in(reg) flags & flags_mask) }
}

fn send_software_interrupt(hartid: usize) {
    let clint_base = 0x200_0000 as *mut u32;

    unsafe {
        clint_base.add(hartid).write_volatile(0x1);
    }
}

fn complete_software_interrupt(hartid: usize) {
    let clint_base = 0x200_0000 as *mut u32;

    unsafe {
        clint_base.add(hartid).write_volatile(0x0);
    }
}

pub fn wake_all_harts() {
    for i in 1..4 {
        println!("waking hart {}", i);
        send_software_interrupt(i);
    }
}

// CLINT Memory Map
// https://sifive.cdn.prismic.io/sifive/b5e7a29c-d3c2-44ea-85fb-acc1df282e21_FU540-C000-v1p3.pdf
pub const MMIO_MTIMECMP: *mut u64 = 0x0200_4000usize as *mut u64;
pub const MMIO_MTIME: *const u64 = 0x0200_BFF8 as *const u64;

pub fn get_mtime() -> u64 {
    unsafe { MMIO_MTIME.read_volatile() }
}

pub fn get_mtimecmp() -> u64 {
    unsafe { MMIO_MTIMECMP.read_volatile() }
}

pub fn schedule_machine_timer_interrupt(quantum: usize) {
    unsafe {
        if crate::ENABLE_PREEMPTION {
            MMIO_MTIMECMP.add(cpu::get_mhartid()).write_volatile(
                MMIO_MTIME
                    .read_volatile()
                    .wrapping_add(cpu::CONTEXT_SWITCH_TIME * quantum as u64),
            );
        }
    }
}

#[no_mangle]
pub fn tong_os_trap(trap_frame: *mut TrapFrame) {
    process::update_running_process_trap_frame(trap_frame);

    let mcause = cpu::get_mcause();
    // Get interrupt bit from mcause
    let is_async = mcause >> 63 & 1 == 1;
    // Get interrupt cause
    let cause = mcause & 0xfff;

    if is_async {
        match cause {
            3 => {
                println!(
                    "Handling asyng software interrupt on hart {}",
                    cpu::get_mhartid()
                );

                loop {}
            }
            7 => {
                crate::get_print_lock().unlock();
                debug!(
                    "Handling async timer interrupt: mcause {}, pid {}",
                    cause,
                    process::get_running_process_pid().unwrap()
                );
                if let Some(_) = process::get_running_process_pid() {
                    process::update_running_process_to_ready();
                }
                scheduler::schedule();
            }
            11 => unsafe {
                debug!("Handling external interrupt!");

                let buffer: &mut alloc::string::String =
                    core::mem::transmute((*trap_frame).regs[GeneralPurposeRegister::A1 as usize]);

                if let Some(external_interrupt) = plic::next() {
                    match external_interrupt {
                        // UART
                        10 => {
                            let mut uart = uart::Uart::new(0x1000_0000);

                            if let Some(c) = uart.get() {
                                match c {
                                    // backspace
                                    8 | 127 => {
                                        // remove last char from buffer
                                        print!("{0} {0}", 8 as char);
                                        buffer.pop();
                                        plic::complete(external_interrupt);
                                        process::switch_to_process(trap_frame);
                                    }
                                    // Enter
                                    10 | 13 => {
                                        println!("");
                                        uart::READING = false;
                                        plic::complete(external_interrupt);

                                        let flags = 1 << 7;
                                        asm!("csrw mie, {}", in(reg) flags);

                                        schedule_machine_timer_interrupt(1);
                                        process::switch_to_process(trap_frame);
                                    }
                                    // Char
                                    _ => {
                                        print!("{}", c as char);
                                        buffer.push(c as char);

                                        plic::complete(external_interrupt);
                                        process::switch_to_process(trap_frame);
                                    }
                                }
                            }
                        }
                        other => panic!(
                            "Unhandled External Interrupt cause: {}, code {}",
                            cause, other
                        ),
                    }
                }
            },
            _ => {
                panic!(
                    "Unhandled async trap CPU#{} -> {}\n",
                    cpu::get_mhartid(),
                    cause
                );
            }
        }
    } else {
        match cause {
            8 => {
                let which_code = unsafe { (*trap_frame).regs[GeneralPurposeRegister::A0 as usize] };

                debug!(
                    "Handling user ecall exception: mcause {}, pid {}, syscall code {}",
                    cause,
                    process::get_running_process_pid().unwrap(),
                    which_code,
                );

                match which_code {
                    // Exiting process
                    0 => {
                        debug!("handling exit");
                        // Check if child process needs to reschedule parent
                        if let Some(blocked) = process::get_running_process_blocking_pid() {
                            process::wake_process(blocked);
                        }
                        process::delete_running_process();
                        scheduler::schedule();
                    }
                    // Create thread
                    1 => {
                        debug!("handling create thread");
                        let process_address =
                            unsafe { (*trap_frame).regs[GeneralPurposeRegister::A1 as usize] };
                        let process_arg =
                            unsafe { (*trap_frame).regs[GeneralPurposeRegister::A2 as usize] };
                        let new_process = process::Process::new(process_address, process_arg);
                        let new_process_pid = new_process.pid;
                        process::process_list_add(new_process);
                        unsafe {
                            (*trap_frame).regs[GeneralPurposeRegister::A0 as usize] =
                                new_process_pid;
                            (*trap_frame).pc += 4;
                        }
                        process::switch_to_process(trap_frame);
                    }
                    // Joining thread
                    2 => {
                        debug!("handling joining");
                        let joining_pid =
                            unsafe { (*trap_frame).regs[GeneralPurposeRegister::A1 as usize] };
                        debug!("joining pid: {}", joining_pid);

                        unsafe {
                            (*trap_frame).pc += 4;
                        };
                        // if joining pid has already exited
                        if !process::process_list_contains(joining_pid) {
                            debug!("not contains");
                            // add runnign to proc list as readdy and schedule
                            process::update_running_process_to_ready();
                            scheduler::schedule();
                        } else {
                            debug!("contains");
                            let blocking_pid = process::get_running_process_pid().unwrap();
                            process::set_blocking_pid(joining_pid, blocking_pid);
                            process::update_running_process_to_blocked();
                            scheduler::schedule();
                        }
                    }
                    // syscall sleep
                    3 => {
                        debug!("handling sleep");
                        unsafe {
                            (*trap_frame).pc += 4;
                        };
                        let amount =
                            unsafe { (*trap_frame).regs[GeneralPurposeRegister::A1 as usize] };
                        let until =
                            get_mtime() as usize + amount * cpu::CONTEXT_SWITCH_TIME as usize;

                        if crate::ENABLE_PREEMPTION {
                            process::update_running_process_to_sleeping(until);
                            scheduler::schedule();
                        } else {
                            // sleep
                            loop {
                                if (get_mtime() as usize) >= until {
                                    break;
                                }
                            }
                            schedule_machine_timer_interrupt(1);
                            process::switch_to_process(trap_frame);
                        }
                    }
                    // syscall input keyboard
                    4 => {
                        debug!("handling input keyboard");
                        unsafe {
                            uart::READING = true;
                        }
                        // UART
                        plic::set_threshold(6);
                        plic::set_priority(10, 7);
                        plic::enable(10);

                        unsafe {
                            // [11] = MEIE (Machine External Interrupt Enable)
                            let flags = 1 << 11;
                            asm!("csrw mie, {}", in(reg) flags);
                        }
                        unsafe {
                            (*trap_frame).pc += 4;
                        }
                        process::switch_to_process(trap_frame);
                    }
                    // syscall print line
                    5 => {
                        debug!("handling print line");
                        unsafe {
                            (*trap_frame).pc += 4;
                        }
                        let buffer: &mut alloc::string::String = unsafe {
                            core::mem::transmute(
                                (*trap_frame).regs[GeneralPurposeRegister::A1 as usize],
                            )
                        };

                        println!("{}", buffer);
                        process::switch_to_process(trap_frame);
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
                    cpu::get_mhartid(),
                    cause,
                    mtval
                );
            }
        }
    }
}
