use crate::lock::Mutex;
use crate::process;
use alloc::format;

const ITERATIONS: isize = 3;
const NUM_PHILOSOPHERS: isize = 5;
const SLEEP_TIME: usize = 500;

static mut TABLE: Mutex = Mutex::new();
static mut CHOPSTICK: [Mutex; NUM_PHILOSOPHERS as usize] =
    [Mutex::new(); NUM_PHILOSOPHERS as usize];

pub unsafe fn philosopher_dinner(n: isize) {
    let first = if n < (NUM_PHILOSOPHERS - 1) { n } else { 0 };
    let second = if n < NUM_PHILOSOPHERS - 1 {
        n + 1
    } else {
        NUM_PHILOSOPHERS - 1
    };

    for i in (0..=ITERATIONS).rev() {
        TABLE.spin_lock();
        process::print_str(&format!("Philosopher {} is thinking. Iteration={}", n, i));
        TABLE.unlock();

        process::sleep(SLEEP_TIME);

        TABLE.spin_lock();
        process::print_str(&format!("Philosopher {} is hungry. Iteration={}", n, i));
        TABLE.unlock();

        CHOPSTICK[first as usize].spin_lock();
        CHOPSTICK[second as usize].spin_lock();

        TABLE.spin_lock();
        process::print_str(&format!("Philosopher {} is eating. Iteration={}", n, i));
        TABLE.unlock();

        process::sleep(SLEEP_TIME);

        TABLE.spin_lock();
        process::print_str(&format!("Philosopher {} is sate. Iteration={}", n, i));
        TABLE.unlock();

        CHOPSTICK[first as usize].unlock();
        CHOPSTICK[second as usize].unlock();
    }

    TABLE.spin_lock();
    process::print_str(&format!("Philosopher {} is done!", { n }));
    TABLE.unlock();

    process::exit();
}

pub unsafe fn main() {
    process::print_str("The Philosopher's Dinner!");

    TABLE.spin_lock();

    let mut philosopher = [0; NUM_PHILOSOPHERS as usize];

    for i in 0..NUM_PHILOSOPHERS {
        process::print_str(&format!("Creating philosopher: {}", i));
        philosopher[i as usize] = process::create_thread(philosopher_dinner as usize, i as usize);
    }

    process::print_str("Philosophers are alive and hungry!");

    process::print_str("The dinner is served ...");
    TABLE.unlock();

    for i in 0..NUM_PHILOSOPHERS {
        let pid = philosopher[i as usize];
        process::join(pid);

        TABLE.spin_lock();
        process::print_str(&format!("Philosopher {} ate {} times!", i, ITERATIONS));
        TABLE.unlock();
    }

    process::print_str("Finished philosophers dinner!");
    process::exit();
}
