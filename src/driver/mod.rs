use gpio::GPIODriver;
use manager::DriverManager;
use uart::UARTDriver;

pub mod gpio;
pub mod manager;
pub mod uart;

pub trait Driver {
    unsafe fn init(&self) -> Result<(), &'static str>;
}

pub struct DriverDescriptor {
    name: &'static str,
    driver: &'static (dyn Driver + Sync),
    post_init: Option<unsafe fn() -> Result<(), &'static str>>,
}

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
            UART_DRIVER.write_str("Hello there");
            Ok(())
        }),
    };

    DRIVER_MANAGER.register_driver(gpio_descriptor);
    DRIVER_MANAGER.register_driver(uart_descriptor);
}

pub fn manager() -> &'static DriverManager<DRIVER_COUNT> {
    &DRIVER_MANAGER
}
