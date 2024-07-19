pub mod mmio {
    pub fn read(addr: u32) -> u32 {
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    pub fn write(addr: u32, val: u32) {
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, val);
        }
    }

    pub fn read_bits(addr: u32, offset: u32, bits: u32) -> u32 {
        let mut val = read(addr);
        val >>= offset;
        let mask = 1 << bits;
        val &= mask - 1;
        val
    }
}
