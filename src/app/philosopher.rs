use crate::lock::Mutex;
use crate::process;

const ITERATIONS: isize = 3;
const NUM_PHILOSOPHERS: isize = 5;
const DELAY_ITERATIONS: isize = 100;

static mut TABLE: Mutex = Mutex::new();
static mut CHOPSTICK: [Mutex; NUM_PHILOSOPHERS as usize] =
    [Mutex::new(); NUM_PHILOSOPHERS as usize];

fn delay(n: isize) -> isize {
    let mut sum = 0;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                sum += i * j + 6 * k;
            }
        }
    }
    sum
}

pub unsafe fn philosopher_dinner(n: isize) {
    let first = if n < (NUM_PHILOSOPHERS - 1) { n } else { 0 };
    let second = if n < NUM_PHILOSOPHERS - 1 {
        n + 1
    } else {
        NUM_PHILOSOPHERS - 1
    };

    for i in (0..=ITERATIONS).rev() {
        TABLE.spin_lock();
        println!("Philosopher {} is thinking. Iteration={}", n, i);
        TABLE.unlock();

        delay(DELAY_ITERATIONS);

        TABLE.spin_lock();
        println!("Philosopher {} is hungry. Iteration={}", n, i);
        TABLE.unlock();

        CHOPSTICK[first as usize].spin_lock();
        CHOPSTICK[second as usize].spin_lock();

        TABLE.spin_lock();
        println!("Philosopher {} is eating. Iteration={}", n, i);
        TABLE.unlock();

        delay(DELAY_ITERATIONS);

        TABLE.spin_lock();
        println!("Philosopher {} is sate. Iteration={}", n, i);
        TABLE.unlock();

        CHOPSTICK[first as usize].unlock();
        CHOPSTICK[second as usize].unlock();
    }

    TABLE.spin_lock();
    println!("done");
    TABLE.unlock();

    process::exit();
}

pub unsafe fn main() {
    println!("The Philosopher's Dinner!");

    TABLE.spin_lock();

    let mut philosopher = [0; NUM_PHILOSOPHERS as usize];

    for i in 0..NUM_PHILOSOPHERS {
        println!("Creating philosopher: {}", i);
        philosopher[i as usize] = process::create_thread(philosopher_dinner as usize, i as usize);
    }

    println!("Philosophers are alive and hungry!");

    println!("The dinner is served ...");
    TABLE.unlock();

    for i in 0..NUM_PHILOSOPHERS {
        let pid = philosopher[i as usize];
        process::join(pid);

        TABLE.spin_lock();
        println!("Philosopher {} ate {} times!", i, ITERATIONS);
        TABLE.unlock();
    }

    println!("Finished philosophers dinner!");
    process::exit();
}
