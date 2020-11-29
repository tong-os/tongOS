use crate::lock::Mutex;
use crate::process;
use alloc::format;

const ITERATIONS: isize = 3;
const NUM_PHILOSOPHERS: usize = 5;
const SLEEP_TIME: usize = 500;

pub unsafe fn philosopher_dinner(n: usize, table: *mut Mutex, chopstick: *mut Mutex) {
    let chopstick = core::slice::from_raw_parts_mut(chopstick, NUM_PHILOSOPHERS);
    let table = &mut (*table);

    let first = if n < (NUM_PHILOSOPHERS - 1) { n } else { 0 };
    let second = if n < NUM_PHILOSOPHERS - 1 {
        n + 1
    } else {
        NUM_PHILOSOPHERS - 1
    };

    for i in (0..=ITERATIONS).rev() {
        table.spin_lock();
        process::print_str(&format!("Philosopher {} is thinking. Iteration={}", n, i));
        table.unlock();

        process::sleep(SLEEP_TIME);

        table.spin_lock();
        process::print_str(&format!("Philosopher {} is hungry. Iteration={}", n, i));
        table.unlock();

        chopstick[first as usize].spin_lock();
        chopstick[second as usize].spin_lock();

        table.spin_lock();
        process::print_str(&format!("Philosopher {} is eating. Iteration={}", n, i));
        table.unlock();

        process::sleep(SLEEP_TIME);

        table.spin_lock();
        process::print_str(&format!("Philosopher {} is sate. Iteration={}", n, i));
        table.unlock();

        chopstick[first as usize].unlock();
        chopstick[second as usize].unlock();
    }

    table.spin_lock();
    process::print_str(&format!("Philosopher {} is done!", { n }));
    table.unlock();

    process::exit();
}

pub fn main() {
    let start_time = process::time_now();
    let mut table = Mutex::new();
    let mut chopstick: [Mutex; NUM_PHILOSOPHERS as usize] =
        [Mutex::new(); NUM_PHILOSOPHERS as usize];

    process::print_str("The Philosopher's Dinner!");

    table.spin_lock();

    let mut philosopher = [0; NUM_PHILOSOPHERS as usize];

    for i in 0..NUM_PHILOSOPHERS {
        process::print_str(&format!("Creating philosopher: {}", i));
        philosopher[i as usize] = process::create_thread(
            philosopher_dinner as usize,
            i as usize,
            &mut table as *mut _ as usize,
            (&mut chopstick).as_mut_ptr() as usize,
        );
    }

    process::print_str("Philosophers are alive and hungry!");

    process::print_str("The dinner is served ...");
    table.unlock();

    for i in 0..NUM_PHILOSOPHERS {
        let pid = philosopher[i as usize];
        process::join(pid);

        table.spin_lock();
        process::print_str(&format!("Philosopher {} ate {} times!", i, ITERATIONS));
        table.unlock();
    }

    let time = process::time_now() - start_time;
    let time = time / crate::cpu::FREQ as usize;
    process::print_str(&format!(
        "Finished philosophers dinner! time elapsed {} seconds.",
        time
    ));
    process::exit();
}
