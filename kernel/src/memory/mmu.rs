use super::translation_table::KERNEL_TABLES;
use crate::{read_reg, write_reg};
use core::arch::asm;

#[allow(dead_code)]
#[derive(Debug)]
pub enum MMUEnableError {
    AlreadyEnabled,
    Granule64KNotSupported,
    Other(&'static str),
}

pub unsafe fn enable() -> Result<(), MMUEnableError> {
    if is_enabled() {
        return Err(MMUEnableError::AlreadyEnabled);
    }

    // Make sure cpu supports 64k granules
    let id_aa64mmfr0 = read_reg!("ID_AA64MMFR0_EL1");
    if (id_aa64mmfr0 & (0b1111 << 24)) != 0 {
        return Err(MMUEnableError::Granule64KNotSupported);
    }

    setup_mair();

    KERNEL_TABLES
        .populate()
        .map_err(|e| MMUEnableError::Other(e))?;

    write_reg!("TTBR0_EL1", KERNEL_TABLES.phys_base_addr());

    configure_tcr();

    // Turn on MMU
    let mut sctlr: u64 = 1; // MMU on
    sctlr |= 1 << 2; // Cacheable memory
    sctlr |= 1 << 12; // Also something about caching
    write_reg!("SCTLR_EL1", sctlr);
    asm!("isb sy");

    Ok(())
}

pub fn is_enabled() -> bool {
    (read_reg!("SCTLR_EL1") & 1) == 1
}

unsafe fn configure_tcr() {
    let mut tcr: u64 = 0;

    let num_bits = (super::map::END_INCLUSIVE + 1).trailing_zeros();
    let t0sz = (64 - num_bits) as u64;
    assert!(t0sz <= 0x3F, "t0sz must fit in 6 bits");

    tcr |= 0b010 << 32; // 40 bit IPA
    tcr |= 0b01 << 14; // 64 Kib Granule size
    tcr |= 0b11 << 12; // Inner sharable
    tcr |= 0b01 << 10; // Outer chachable normal memory
    tcr |= 0b01 << 8; // Inner chachable normal memory
    tcr |= 0b1 << 23; // Disable TTBR1 Walks
    tcr |= t0sz & 0x3F;

    write_reg!("TCR_EL1", tcr);
}

fn setup_mair() {
    // Attribute 1 - Cacheable DRAM
    let attrib1: u64 = 0b1111_1111; // Write-back non-transistent, alloc read and write for inner and outer

    // Attribute 0 - Device memory
    let attrib0: u64 = 0b0000_0100; // non-gathering non-reordering non-cacheable

    write_reg!("MAIR_EL1", (attrib0 | (attrib1 << 8)));
}
