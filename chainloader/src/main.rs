#![feature(format_args_nl)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uart::Uart;

mod boot;
mod gpio;
mod mmio;
mod uart;

pub static UART: Uart = Uart::new();

unsafe fn kernel_init() -> ! {
    gpio::map_uart();
    UART.init();

    kernel_start()
}

const BLOCK_SIZE: usize = 512;

fn kernel_start() -> ! {
    write!(UART, "Requesting binary\n");
    write!(UART, "\0\0\0");

    let mut kernel_size = [0; 4];
    for i in 0..4 {
        kernel_size[i] = UART.read_blocking();
    }
    let kernel_size = u32::from_le_bytes(kernel_size);
    let blocks = kernel_size / BLOCK_SIZE as u32;

    let kernel_addr = 0x80000 as *mut u8;
    for b in 0..blocks {
        let mut verify = [0; BLOCK_SIZE];
        loop {
            for v in &mut verify {
                *v = UART.read_blocking();
            }

            for v in &verify {
                UART.write(*v)
            }
            if UART.read_blocking() == b'G' {
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
    let _ = write!(UART, "KERNEL PANIC!: {}\n", info.message());
    if let Some(loc) = info.location() {
        let _ = write!(UART, "in file {}:{}", loc.file(), loc.line());
    }

    loop {}
}
