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

    fn write(&mut self, c: u8) {
        while mmio::read_bits(FR, FR_TXFF, 1) == 1 {
            unsafe {
                asm!("nop");
            }
        }

        mmio::write(DR, c as u32);
    }

    fn read_blocking(&self) -> u8 {
        while mmio::read_bits(FR, FR_RXFE, 1) == 1 {
            unsafe {
                asm!("nop");
            }
        }

        match mmio::read(DR) as u8 {
            b'\r' => b'\n',
            c => c,
        }
    }

    fn read(&self) -> Option<u8> {
        if mmio::read_bits(FR, FR_RXFE, 1) == 1 {
            return None;
        }
        Some(self.read_blocking())
    }
}

impl Write for UARTDriverInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.write(c);
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
        let _ = self.inner.lock(|i| {
            let mut b = [0; 4];
            let s = c.encode_utf8(&mut b);
            for c in s.bytes() {
                i.write(c);
            }
        });
    }
    pub fn write_str(&self, s: &str) {
        let _ = self.inner.lock(|i| i.write_str(s));
    }

    /// Block until the transmit FIFO is empty
    pub fn flush(&self) {
        self.inner.lock(|i| i.flush());
    }

    /// Block until a u8 is sent
    pub fn read_blocking(&self) -> u8 {
        self.inner.lock(|i| i.read_blocking())
    }
    /// Try to read a u8 without blocking
    pub fn read(&self) -> Option<u8> {
        self.inner.lock(|i| i.read())
    }

    /// Block until a char is sent
    pub fn read_char_blocking(&self) -> char {
        self.inner.lock(|i| {
            let mut b = [0; 4];
            for idx in 0..4 {
                b[idx] = i.read_blocking();

                if let Ok(c) = core::str::from_utf8(&b[0..idx]) {
                    // TODO: Check that str has lenght of 1?
                    return c.chars().next().unwrap();
                }
            }

            // TODO: Return an error
            char::REPLACEMENT_CHARACTER
        })
    }
    /// Try to read a char
    /// If a multi byte char is read will block until all bytes are read
    pub fn read_char(&self) -> Option<char> {
        self.inner.lock(|i| {
            let mut b = [0; 4];
            b[0] = i.read()?;
            if let Ok(c) = core::str::from_utf8(&b[0..=1]) {
                // TODO: Check that str has lenght of 1?
                return Some(c.chars().next().unwrap());
            }

            for idx in 1..4 {
                b[idx] = i.read_blocking();

                if let Ok(c) = core::str::from_utf8(&b[0..=idx]) {
                    // TODO: Check that str has lenght of 1?
                    return Some(c.chars().next().unwrap());
                }
            }

            // TODO: Return an error
            None
        })
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
