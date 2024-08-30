use super::translation_table::KERNEL_TABLES;
use aarch64_cpu::{asm::barrier, registers::*};

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

    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
        return Err(MMUEnableError::Granule64KNotSupported);
    }

    setup_mair();

    KERNEL_TABLES
        .populate()
        .map_err(|e| MMUEnableError::Other(e))?;

    TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_addr());

    configure_tcr();

    // Turn on MMU
    SCTLR_EL1.write(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);

    Ok(())
}

pub fn is_enabled() -> bool {
    SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
}

unsafe fn configure_tcr() {
    let num_bits = (super::map::END_INCLUSIVE + 1).trailing_zeros();
    let t0sz = (64 - num_bits) as u64;

    TCR_EL1.write(
        TCR_EL1::IPS::Bits_40
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::SH0::Inner
            + TCR_EL1::TG0::KiB_64
            + TCR_EL1::EPD1::DisableTTBR1Walks
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::T0SZ.val(t0sz),
    );
}

fn setup_mair() {
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc,
    );
}
