use crate::{gpio, uart};

pub unsafe fn kernel_init() -> ! {
    gpio::map_uart();
    uart::init();

    kernel_start()
}

fn kernel_start() -> ! {
    uart::write_str("Hello world!\n");

    loop {
        if let Some(c) = uart::read_char() {
            uart::write_char(c);
        }
    }
}
