use crate::kernel_init;
use aarch64_cpu::registers::*;
use core::arch::{asm, global_asm};

global_asm!(include_str!("boot.S"));

#[no_mangle]
pub unsafe extern "C" fn __start_rust(sp_addr: u64) -> ! {
    prepare_jump_to_el1(sp_addr);

    // Jumps to address in ELR
    asm!("eret", options(noreturn))
}

unsafe fn prepare_jump_to_el1(sp_addr: u64) {
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
    CNTVOFF_EL2.set(0);

    CPACR_EL1
        .write(CPACR_EL1::FPEN::TrapNothing + CPACR_EL1::TTA::NoTrap + CPACR_EL1::ZEN::TrapNothing);
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    SPSR_EL2.write(
        SPSR_EL2::F::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::D::Masked
            + SPSR_EL2::M::EL1h,
    );

    ELR_EL2.set(kernel_init as *const () as u64);
    SP_EL1.set(sp_addr);
}
