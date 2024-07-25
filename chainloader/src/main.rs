#![feature(format_args_nl)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use lib::driver::{gpio::GPIODriver, manager::DriverManager, uart::UARTDriver, DriverDescriptor};
mod boot;

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
        post_init: Some(|| Ok(())),
    };

    DRIVER_MANAGER.register_driver(gpio_descriptor);
    DRIVER_MANAGER.register_driver(uart_descriptor);
}

unsafe fn kernel_init() -> ! {
    setup_drivers();
    DRIVER_MANAGER.init();

    kernel_start()
}

const BLOCK_SIZE: usize = 512;

fn kernel_start() -> ! {
    UART_DRIVER.write_str("Requesting binary\n");
    UART_DRIVER.write_str("\0\0\0");

    let mut kernel_size = [0; 4];
    for i in 0..4 {
        kernel_size[i] = UART_DRIVER.read_blocking();
    }
    let kernel_size = u32::from_le_bytes(kernel_size);
    let blocks = kernel_size / BLOCK_SIZE as u32;

    let kernel_addr = 0x80000 as *mut u8;
    for b in 0..blocks {
        let mut verify = [0; BLOCK_SIZE];
        loop {
            for v in &mut verify {
                *v = UART_DRIVER.read_blocking();
            }

            for v in &verify {
                UART_DRIVER.write_byte(*v)
            }
            if UART_DRIVER.read_blocking() == b'G' {
                break;
            }
        }

        for (i, c) in verify.iter().enumerate() {
            unsafe {
                core::ptr::write_volatile(
                    kernel_addr.offset((b * BLOCK_SIZE as u32 + i as u32) as isize),
                    *c,
                );
            }
        }
    }

    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
    kernel()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let uart = UARTDriver::new();
    let _ = write!(uart, "KERNEL PANIC!: {}\n", info.message());
    if let Some(loc) = info.location() {
        let _ = write!(uart, "in file {}:{}", loc.file(), loc.line());
    }

    loop {}
}
