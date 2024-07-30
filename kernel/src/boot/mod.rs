use crate::kernel::kernel_init;
use core::arch::{asm, global_asm};

global_asm!(include_str!("boot.S"));

#[no_mangle]
pub unsafe extern "C" fn __start_rust(sp_addr: u64) -> ! {
    prepare_jump_to_el1(sp_addr);

    // Jumps to address in ELR
    asm!("eret", options(noreturn))
}

unsafe fn prepare_jump_to_el1(sp_addr: u64) {
    // Don't trap access to timer counter registers in EL1 or EL0
    let cnthctl: u64 = 0b11;
    asm!("msr CNTHCTL_EL2, {}", in(reg) cnthctl);

    // Set timer counter offset to 0
    let cntvoff: u64 = 0;
    asm!("msr CNTVOFF_EL2, {}", in(reg) cntvoff);

    // Don't trap accesses to SVE, Advanced SIMD, and
    // floating-point registers in EL0 or EL1
    let cpacr: u64 = 0b11 << 20;
    asm!("msr CPACR_EL1, {}", in(reg) cpacr);

    // Run in aarch64 mode, not aarch32
    let hcr: u64 = 1 << 31;
    asm!("msr HCR_EL2, {}", in(reg) hcr);

    // Mask all asynchronous interrupts
    let mut spsr: u64 = 0b1111 << 6;
    // Use SP_EL1 as the stack pointer
    spsr |= 0b0101;
    asm!("msr SPSR_EL2, {}", in(reg) spsr);

    // Set address to jump after 'eret'
    let elr = kernel_init as *const () as u64;
    asm!("msr ELR_EL2, {}", in(reg) elr);

    // Reuse the stack pointer of EL2 for EL1, since we never return
    // to EL2
    asm!("msr SP_EL1, {}", in(reg) sp_addr);
}
