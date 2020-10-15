use tong_os::assembly::*;
use tong_os::assignment;
use tong_os::{print, println};

#[no_mangle]
extern "C" fn kinit(_hartid: usize) -> ! {
    tong_os::uart::Uart::new(0x1000_0000).init();
    print!("Set all bytes in BSS to zero ...");
    for address in unsafe { BSS_START..BSS_END } {
        unsafe {
            (address as *mut usize).write_volatile(0);
        }
    }
    println!("Finished!");

    assignment::test_bss();

    println!("Init pages");
    tong_os::page::init();
    println!("Finished!");

    println!("You are now in ...");
    println!(concat!(
        " _                    _____ _____ \n",
        "| |                  |  _  /  ___|\n",
        "| |_ ___  _ __   __ _| | | \\ `--. \n",
        "| __/ _ \\| '_ \\ / _` | | | |`--. \\\n",
        "| || (_) | | | | (_| \\ \\_/ /\\__/ /\n",
        " \\__\\___/|_| |_|\\__, |\\___/\\____/ \n",
        "                 __/ | \n",
        "                |___/ ",
    ));
    println!("Init tests!");
    assignment::print_sections();

    loop {}
}
