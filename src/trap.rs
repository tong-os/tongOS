use crate::process::{self, Process};
use crate::scheduler::schedule;

pub fn init() {
    use crate::assembly::__tong_os_trap;
    unsafe {
        asm!("csrw mtvec, {}", in(reg) (__tong_os_trap as usize));
    }
}

#[no_mangle]
pub fn tong_os_trap(process: &Process) {
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
                println!("Handling exception 8 to process#{}", process.pid);
                process::print_process_list();
                process::process_list_remove(process.pid);
                println!("Process list after remove: ");
                process::print_process_list();

                if let Some(next_process) = schedule() {
                    process::switch_to_user(&next_process);
                } else {
                    panic!("Next process not found!");
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
