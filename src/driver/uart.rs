use super::Driver;
use crate::{memory::mmio, sync::NullLock};
use core::{arch::asm, fmt::Write};

const BASE: u32 = 0xFE201000;
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

struct UARTDriverInner {}
impl UARTDriverInner {
    fn init(&self) {
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

    fn flush(&self) {
        while mmio::read_bits(FR, FR_TXFF, 1) == 1 {
            unsafe {
                asm!("nop");
            }
        }
    }

    fn read_char_blocking(&self) -> char {
        while mmio::read_bits(FR, FR_RXFE, 1) == 1 {
            unsafe {
                asm!("nop");
            }
        }

        match mmio::read(DR) as u8 as char {
            '\r' => '\n',
            c => c,
        }
    }

    fn read_char(&self) -> Option<char> {
        if mmio::read_bits(FR, FR_RXFE, 1) == 1 {
            return None;
        }
        Some(self.read_char_blocking())
    }
}

impl Write for UARTDriverInner {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        while mmio::read_bits(FR, FR_TXFF, 1) == 1 {
            unsafe {
                asm!("nop");
            }
        }

        mmio::write(DR, c as u32);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
}

pub struct UARTDriver {
    inner: NullLock<UARTDriverInner>,
}
impl UARTDriver {
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(UARTDriverInner {}),
        }
    }

    pub fn write_char(&self, c: char) {
        // Cannot return an error
        let _ = self.inner.lock(|i| i.write_char(c));
    }
    pub fn write_str(&self, s: &str) {
        let _ = self.inner.lock(|i| i.write_str(s));
    }
    /// Block until the transmit FIFO is empty
    pub fn flush(&self) {
        self.inner.lock(|i| i.flush());
    }
    /// Block until a char is sent
    pub fn read_char_blocking(&self) -> char {
        self.inner.lock(|i| i.read_char_blocking())
    }
    /// Try to read a char without blocking
    pub fn read_char(&self) -> Option<char> {
        self.inner.lock(|i| i.read_char())
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result {
        self.inner.lock(|i| i.write_fmt(args))
    }
}
impl Driver for UARTDriver {
    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|i| i.init());
        Ok(())
    }
}

