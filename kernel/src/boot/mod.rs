use crate::kernel_init;
use core::arch::{asm, global_asm};

use crate::write_reg;

global_asm!(include_str!("boot.S"));

#[no_mangle]
pub unsafe extern "C" fn __start_rust(sp_addr: u64) -> ! {
    prepare_jump_to_el1(sp_addr);

    // Jumps to address in ELR
    asm!("eret", options(noreturn))
}

unsafe fn prepare_jump_to_el1(sp_addr: u64) {
    // Don't trap access to timer counter registers in EL1 or EL0
    write_reg!("CNTHCTL_EL2", 0b11);

    // Set timer counter offset to 0
    write_reg!("CNTVOFF_EL2", 0);

    // Don't trap accesses to SVE, Advanced SIMD, and
    // floating-point registers in EL0 or EL1
    write_reg!("CPACR_EL1", 0b11 << 20);

    // Run in aarch64 mode, not aarch32
    write_reg!("HCR_EL2", 1 << 31);

    // Mask all asynchronous interrupts
    let mut spsr: u64 = 0b1111 << 6;
    // Use SP_EL1 as the stack pointer
    spsr |= 0b0101;
    write_reg!("SPSR_EL2", spsr);

    // Set address to jump after 'eret'
    write_reg!("ELR_EL2", kernel_init as *const () as u64);

    // Reuse the stack pointer of EL2 for EL1, since we never return to EL2
    write_reg!("SP_EL1", sp_addr);
}
