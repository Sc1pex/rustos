use crate::driver;
use lib::{exception::current_el, info, memory::mmu};

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

    // info!("Spinning for 2 seconds");
    // time::spin_for(Duration::from_secs(2));
    // info!("Done");
    //
    // time::spin_for(Duration::from_nanos(10));

    loop {
        if let Some(c) = driver::UART_DRIVER.read_char() {
            info!("Read {}", c)
        }
    }
}
