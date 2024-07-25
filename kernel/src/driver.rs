use lib::{
    driver::{gpio::GPIODriver, manager::DriverManager, uart::UARTDriver, DriverDescriptor},
    log,
};

pub const DRIVER_COUNT: usize = 2;
static DRIVER_MANAGER: DriverManager<DRIVER_COUNT> = DriverManager::new();

pub static GPIO_DRIVER: GPIODriver = GPIODriver::new();
pub static UART_DRIVER: UARTDriver = UARTDriver::new();

pub unsafe fn setup_drivers() {
    let gpio_descriptor = DriverDescriptor {
        name: "GPIO",
        driver: &GPIO_DRIVER,
        post_init: Some(|| {
            GPIO_DRIVER.map_uart();
            Ok(())
        }),
    };

    let uart_descriptor = DriverDescriptor {
        name: "UART",
        driver: &UART_DRIVER,
        post_init: Some(|| {
            log::logger().set_writer(&UART_DRIVER);
            Ok(())
        }),
    };

    DRIVER_MANAGER.register_driver(gpio_descriptor);
    DRIVER_MANAGER.register_driver(uart_descriptor);
}

pub fn manager() -> &'static DriverManager<DRIVER_COUNT> {
    &DRIVER_MANAGER
}
