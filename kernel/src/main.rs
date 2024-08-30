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
mod sys_reg;
mod time;

pub unsafe fn kernel_init() -> ! {
    #[cfg(feature = "debug_wait")]
    core::arch::asm!("1:", "wfe", "b 1b");

    mmu::enable().unwrap();

    driver::setup_drivers();
    driver::manager().init();

    kernel_start()
}

fn kernel_start() -> ! {
    info!("Kernel started");
    info!("Current privilege level: {:?}", current_el());

    memory::layout::print_kernel_memory_layout();

    info!("Spinning for 1 seconds");
    time::spin_for(Duration::from_secs(2));

    loop {
        if let Some(c) = driver::UART_DRIVER.read_char() {
            info!("Read {}", c)
        }
    }
}
