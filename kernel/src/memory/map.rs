use core::cell::UnsafeCell;

extern "Rust" {
    static __code_start: UnsafeCell<()>;
    static __code_end: UnsafeCell<()>;
}

pub(super) const END_INCLUSIVE: usize = 0xFFFF_FFFF;

pub mod mmio {
    pub const START: usize = 0xFE00_0000;
    pub const END_INCLUSIVE: usize = 0xFF84_FFFF;

    pub const GPIO_START: usize = START + 0x20_0000;
    pub const UART0_START: usize = START + 0x20_1000;
}

#[inline(always)]
pub(super) fn code_start() -> usize {
    unsafe { __code_start.get() as usize }
}

#[inline(always)]
pub(super) fn code_end_exclusize() -> usize {
    unsafe { __code_end.get() as usize }
}
