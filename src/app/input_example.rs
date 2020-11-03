
use crate::process;
use alloc::string::String;

pub unsafe fn main() {
    let mut my_name = String::new();
    let mut my_year = String::new();

    println!("Welcome to the simple external interrupt tester!");
    println!("What is your name?");
    process::read_line(&mut my_name);

    println!("What year were you born?");
    process::read_line(&mut my_year);

    let age = 2020 - my_year.parse::<i32>().unwrap();
    println!("Hello {}, who has born in {}.\nYou are now {} years old!", my_name, my_year, age);
    println!("I'm going to sleep now!");
    process::sleep(age as usize);
    println!("I'm back.");
    process::exit();
}
