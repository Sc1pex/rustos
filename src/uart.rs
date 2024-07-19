use crate::memory::mmio;
use core::arch::asm;

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

pub fn init() {
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

pub fn write_char(c: char) {
    while mmio::read_bits(FR, FR_TXFF, 1) == 1 {
        unsafe {
            asm!("nop");
        }
    }

    mmio::write(DR, c as u32);
}

pub fn flush() {
    while mmio::read_bits(FR, FR_TXFF, 1) == 1 {
        unsafe {
            asm!("nop");
        }
    }
}

pub fn read_char_blocking() -> char {
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

pub fn read_char() -> Option<char> {
    if mmio::read_bits(FR, FR_RXFE, 1) == 1 {
        return None;
    }
    Some(read_char_blocking())
}

pub fn write_str(s: &str) {
    for c in s.chars() {
        write_char(c);
    }
}
