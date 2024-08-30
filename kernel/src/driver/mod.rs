use core::ops::Deref;

use crate::{log, memory};
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
    pub name: &'static str,
    pub driver: &'static (dyn Driver + Sync),
    pub post_init: Option<unsafe fn() -> Result<(), &'static str>>,
}

pub const DRIVER_COUNT: usize = 2;
static DRIVER_MANAGER: DriverManager<DRIVER_COUNT> = DriverManager::new();

static GPIO_DRIVER: GPIODriver = GPIODriver::new(memory::map::mmio::GPIO_START);
pub static UART_DRIVER: UARTDriver = UARTDriver::new(memory::map::mmio::UART0_START);

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

struct MMIOWrapper<T> {
    addr: usize,
    _t: core::marker::PhantomData<fn() -> T>,
}

impl<T> MMIOWrapper<T> {
    const fn new(addr: usize) -> Self {
        Self {
            addr,
            _t: core::marker::PhantomData,
        }
    }
}

impl<T> Deref for MMIOWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const _) }
    }
}
