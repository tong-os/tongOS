use crate::lock::Mutex;
use crate::process::{self, Process};

const iterations: isize = 3;
const num_philosophers: isize = 5;
const delay_iterations: isize = 100;

static mut table: Mutex = Mutex::new();
static mut chopstick: [Mutex; num_philosophers as usize] =
    [Mutex::new(); num_philosophers as usize];

fn create_thread(i: usize) -> usize {
    let proc = process::Process::new(philosopher_dinner as usize, i);
    let pid = proc.pid;
    process::process_list_add(proc);
    
    pid
}

fn delay(n: isize) {
    let mut sum = 0;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                sum += i * j + 6 * k;
            }
        }
    }
}

pub unsafe fn philosopher_dinner(n: isize) {
    let first = if n < (num_philosophers - 1) { n } else { 0 };
    let second = if n < num_philosophers - 1 {
        n + 1
    } else {
        num_philosophers - 1
    };

    for i in (0..=iterations).rev() {
        table.spin_lock();
        println!("Philosopher {} is thinking. Iteration={}", n, i);
        table.unlock();

        delay(delay_iterations);

        table.spin_lock();
        println!("Philosopher {} is hungry. Iteration={}", n, i);
        table.unlock();

        chopstick[first as usize].spin_lock();
        chopstick[second as usize].spin_lock();

        table.spin_lock();
        println!("Philosopher {} is eating. Iteration={}", n, i);
        table.unlock();

        delay(delay_iterations);

        table.spin_lock();
        println!("Philosopher {} is sate. Iteration={}", n, i);
        table.unlock();

        chopstick[first as usize].unlock();
        chopstick[second as usize].unlock();
    }

    table.spin_lock();
    println!("done");
    table.unlock();

    process::exit();
}

pub unsafe fn main() {
    println!("The Philosopher's Dinner!");

    table.spin_lock();

    let mut philosopher = [0; num_philosophers as usize];

    for i in 0..num_philosophers {
        println!("Creating philosopher: {}", i);
        philosopher[i as usize] = create_thread(i as usize);
    }

    println!("Philosophers are alive and hungry!");

    println!("The dinner is served ...");
    table.unlock();

    for i in 0..num_philosophers {
        let pid = philosopher[i as usize];
        process::join(pid);

        table.spin_lock();
        println!("Philosopher {} ate {} times!", i, iterations);
        table.unlock();
    }

    println!("Finished philosophers dinner!");
    process::exit();
}
