mod layout;
pub mod map;
pub mod mmu;
mod translation_table;

pub use layout::print_kernel_memory_layout;

pub mod mmio {
    pub fn read(addr: usize) -> u32 {
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    pub fn write(addr: usize, val: u32) {
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, val);
        }
    }

    pub fn read_bits(addr: usize, offset: u32, bits: u32) -> u32 {
        let mut val = read(addr);
        val >>= offset;
        let mask = 1 << bits;
        val &= mask - 1;
        val
    }
}
