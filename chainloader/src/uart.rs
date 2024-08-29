use crate::mmio;
use core::{
    arch::asm,
    cell::UnsafeCell,
    fmt::{Arguments, Write},
};

const BASE: u32 = 0xFE20_1000;
const DR: u32 = BASE + 0;
const FR: u32 = BASE + 0x18;
const IBRD: u32 = BASE + 0x24;
const FBRD: u32 = BASE + 0x28;
const LCRH: u32 = BASE + 0x2C;
const CR: u32 = BASE + 0x30;
const ICR: u32 = BASE + 0x44;

const LCRH_FEN: u32 = 4;
const LCRH_WLEN: u32 = 5;

const CR_EN: u32 = 0;
const CR_TXE: u32 = 8;
const CR_RXE: u32 = 9;

const FR_RXFE: u32 = 4;
const FR_TXFF: u32 = 5;

fn init() {
    mmio::write(CR, 0); // Disable UART
    mmio::write(ICR, 0); // Clear pending interrups

    // Set baudrate to 921600
    mmio::write(IBRD, 3);
    mmio::write(FBRD, 16);

    // Enable FIFOS, 8 bit world length, no parity
    mmio::write(LCRH, (1 << LCRH_FEN) | (0b11 << LCRH_WLEN));

    // Enable UART, UART TX and RX
    mmio::write(CR, (1 << CR_EN) | (1 << CR_TXE) | (1 << CR_RXE));
}

fn write(c: u8) {
    while mmio::read_bits(FR, FR_TXFF, 1) == 1 {
        unsafe {
            asm!("nop");
        }
    }

    mmio::write(DR, c as u32);
}

fn read_blocking() -> u8 {
    while mmio::read_bits(FR, FR_RXFE, 1) == 1 {
        unsafe {
            asm!("nop");
        }
    }

    mmio::read(DR) as u8
}

/// Exists because `core::fmt::Write` functions require `&mut self` and uart instance is static
/// And static mut requires unsafe everywhere
pub struct UartInner;
pub struct Uart {
    i: UnsafeCell<UartInner>,
}
unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

impl core::fmt::Write for UartInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            write(c);
        }

        Ok(())
    }
}

impl Uart {
    pub const fn new() -> Self {
        Self {
            i: UnsafeCell::new(UartInner),
        }
    }

    pub fn init(&self) {
        init();
    }

    pub fn write_fmt(&self, args: Arguments) {
        let i = unsafe { &mut *self.i.get() };
        let _ = i.write_fmt(args);
    }

    pub fn read_blocking(&self) -> u8 {
        read_blocking()
    }

    pub fn write(&self, c: u8) {
        write(c)
    }
}
