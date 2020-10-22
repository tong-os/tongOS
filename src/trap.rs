use crate::process::Process;
use crate::scheduler::schedule;

pub fn init() {
    unsafe {
        asm!("csrw mtvec, {}", in(reg) (crate::assembly::__tong_os_trap as usize));
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
                crate::process::print_process_list();
                crate::process::process_list_remove(process.pid);
                println!("Process list after remove: ");
                crate::process::print_process_list();

                if let Some(next_process) = schedule() {
                    crate::process::switch_to_user(&next_process);
                } else {
                    panic!("Next process not found!");
                }
            }
            _ => {
                panic!(
                    "Unhandled sync trap CPU#{} -> {}\n",
                    process.context.hartid, cause
                );
            }
        }
    }
}
