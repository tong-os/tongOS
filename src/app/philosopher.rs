use crate::lock::Mutex;
use crate::process::{self, Process};

const iterations: isize = 10;
const num_philosophers: isize = 5;
const delay_iterations: isize = 1000000;

static mut table: Mutex = Mutex::new();
static mut chopstick: [Mutex; num_philosophers as usize] =
    [Mutex::new(); num_philosophers as usize];

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

    for _ in (0..=iterations).rev() {
        table.spin_lock();
        println!("Philosopher {} is thinking", n);
        table.unlock();

        delay(delay_iterations);

        table.spin_lock();
        println!("Philosopher {} is hungry", n);
        table.unlock();

        chopstick[first as usize].spin_lock();
        chopstick[second as usize].spin_lock();

        table.spin_lock();
        println!("Philosopher {} is eating", n);
        table.unlock();

        delay(delay_iterations);

        table.spin_lock();
        println!("Philosopher {} is sate", n);
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

    // let mut philosopher: [Option<Process>; num_philosophers as usize] = [None; num_philosophers as usize];

    for i in 0..num_philosophers {
        // philosopher[i as usize].replace(process::Process::new(philosopher_dinner as usize, 1));
    }

    println!("Philosophers are alive and hungry!");

    println!("The dinner is served ...");
    table.unlock();

    for i in 0..num_philosophers {
        // if let Some(&process) = philosopher[i as usize] {
        //     process.join();
        // }
        table.spin_lock();
        println!("Philosopher {} ate {} times!", i, iterations);
        table.unlock();
    }

    process::exit();
}
