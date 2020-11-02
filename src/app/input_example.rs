
use crate::process;
use alloc::string::String;

pub unsafe fn main() {
    let mut buffer = String::new();

    println!("What is your name?");

    process::input_keyboard(&mut buffer);

    println!("Hello {}", buffer);

    process::exit();
}
