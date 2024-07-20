use crate::driver;

pub unsafe fn kernel_init() -> ! {
    #[cfg(feature = "debug_wait")]
    core::arch::asm!("1:", "wfe", "b 1b");

    driver::setup_drivers();
    driver::manager().init();

    kernel_start()
}

fn kernel_start() -> ! {
    loop {
        if let Some(c) = driver::UART_DRIVER.read_char() {
            driver::UART_DRIVER.write_char(c);
        }
    }
}
