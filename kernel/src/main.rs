#![feature(format_args_nl)]
#![feature(new_range_api)]
#![no_std]
#![no_main]

use core::time::Duration;

use exception::current_el;
use memory::mmu;

mod boot;
mod driver;
mod exception;
mod log;
mod memory;
mod panic;
mod sync;
mod time;

unsafe fn kernel_init() -> ! {
    #[cfg(feature = "debug_wait")]
    core::arch::asm!("1:", "wfe", "b 1b");

    exception::init_handlers();
    mmu::enable().unwrap();

    driver::setup_drivers();
    driver::manager().init();

    kernel_start()
}

fn kernel_start() -> ! {
    info!("Kernel started");
    info!("Current privilege level: {:?}", current_el());

    memory::print_kernel_memory_layout();

    info!("Spinning for 1 seconds");
    time::spin_for(Duration::from_secs(1));

    info!("Trying to read from address 8 GiB...");
    let big_addr: u64 = 8 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    loop {
        if let Some(c) = driver::UART_DRIVER.read_char() {
            info!("Read {}", c)
        }
    }
}
