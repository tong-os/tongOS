use crate::process;
use alloc::format;
use alloc::string::String;

pub unsafe fn main() {
    let mut my_name = String::new();
    let mut my_year = String::new();

    process::print_str("Welcome to the simple external interrupt tester!");
    process::print_str("What is your name?");
    process::read_line(&mut my_name);

    process::print_str("What year were you born?");
    process::read_line(&mut my_year);

    let age = 2020 - my_year.parse::<i32>().unwrap();
    process::print_str(&format!(
        "Hello {}, who has born in {}.\nYou are now {} years old!",
        my_name, my_year, age
    ));
    process::print_str("I'm going to sleep now!");
    process::sleep(age as usize);
    process::print_str("I'm back.");
    process::exit();
}
