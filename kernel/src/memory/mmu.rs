use crate::{read_reg, write_reg};
use core::{arch::asm, range::RangeInclusive};

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

pub fn is_enabled() -> bool {
    (read_reg!("SCTLR_EL1") & 1) == 1
}

fn setup_mair() {
    // Attribute 1 - Cacheable DRAM
    let attrib1: u64 = 0b1111_1111; // Write-back non-transistent, alloc read and write for inner
                                    // and outer

    // Attribute 0 - Device memory
    let attrib0: u64 = 0b0000_0100; // non-gathering non-reordering non-cacheable

    write_reg!("MAIR_EL1", (attrib0 | (attrib1 << 8)));
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PageDescriptor {
    value: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TableDescriptor {
    value: u64,
}

impl PageDescriptor {
    const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    fn from_addr(addr: usize, attribs: AttributeFields) -> Self {
        // Valid page
        let mut value = 0b11;

        match attribs.mem_attributes {
            MemAttributes::CacheableDRAM => {
                // Inner sharable
                value |= 0b11 << 8;
                // Attribute 1: Normal memory
                value |= 1 << 2;
            }
            MemAttributes::Device => {
                // Outer sharable
                value |= 0b10 << 8;
                // Attribute 0: Device memory
                value |= 0 << 2;
            }
        }

        match attribs.acc_perms {
            AccessPermissions::ReadOnly => {
                // Read only, only accesible from EL1
                value |= 0b10 << 6;
            }
            AccessPermissions::ReadWrite => {
                // Read/Write, only accesible from EL1
                value |= 0b00 << 6;
            }
        }

        // Default is allow execute so else is not needed
        if attribs.execute_never {
            value |= 1 << 53;
        }

        // Don't allow execution from EL0
        value |= 1 << 54;

        let shifted = (addr >> SHIFT_64K) as u64;
        value |= (shifted & 0xFFFF_FFFF) << 16;

        // Access flag
        value |= 1 << 10;

        Self { value }
    }
}

impl TableDescriptor {
    const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    fn from_addr(next_table_addr: usize) -> Self {
        let mut value: u64 = 0b11;

        // Each lvl2 entry points to a lvl3 table which is 64kib
        let shifted = (next_table_addr >> SHIFT_64K) as u64;
        value |= (shifted & 0xFFFF_FFFF) << 16;

        Self { value }
    }
}

#[repr(C)]
#[repr(align(65536))]
struct TranslationTables<const TABLES: usize> {
    lvl3: [[PageDescriptor; 1 << 13]; TABLES],

    lvl2: [TableDescriptor; TABLES],
}

impl<const TABLES: usize> TranslationTables<TABLES> {
    const fn new() -> Self {
        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 1 << 13]; TABLES],
            lvl2: [TableDescriptor::new_zeroed(); TABLES],
        }
    }

    fn populate(&mut self) -> Result<(), &'static str> {
        for (i, l2_entry) in self.lvl2.iter_mut().enumerate() {
            let addr = &self.lvl3[i] as *const _ as usize;
            *l2_entry = TableDescriptor::from_addr(addr);

            for (j, l3_entry) in self.lvl3[i].iter_mut().enumerate() {
                let virt_addr = (i << SHIFT_512M) + (j << SHIFT_64K);

                let (output, attribs) = KERNEL_LAYOUT.virt_addr_props(virt_addr)?;

                *l3_entry = PageDescriptor::from_addr(output, attribs);
            }
        }

        Ok(())
    }

    fn phys_base_addr(&self) -> u64 {
        let s = &self.lvl2;
        s as *const _ as u64
    }
}

const KERNEL_LV2_TABLES: usize = (super::map::END_INCLUSIVE + 1) >> SHIFT_512M;
static mut KERNEL_TABLES: TranslationTables<KERNEL_LV2_TABLES> = TranslationTables::new();
static KERNEL_LAYOUT: KernelVirtualLayout<4> = KernelVirtualLayout {
    max_virt_addr: super::map::END_INCLUSIVE,

    layouts: [
        TranslationDescriptor {
            name: "Kernel code and RO data",
            virtual_range: || RangeInclusive {
                start: super::map::code_start(),
                end: super::map::code_end_exclusize() - 1,
            },
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadOnly,
                execute_never: false,
            },
            map_to: None,
        },
        TranslationDescriptor {
            name: "Device MMIO",
            virtual_range: || RangeInclusive {
                start: super::map::mmio::START,
                end: super::map::mmio::END_INCLUSIVE,
            },
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
            map_to: None,
        },
        TranslationDescriptor {
            name: "Remmaped MMIO",
            virtual_range: || RangeInclusive {
                start: 0x1FFF_0000,
                end: 0x1FFF_FFFF,
            },
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
            map_to: Some(super::map::mmio::START + 0x20_0000),
        },
        TranslationDescriptor {
            name: "Other memory",
            virtual_range: || RangeInclusive {
                start: 0,
                end: super::map::END_INCLUSIVE,
            },
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
            map_to: None,
        },
    ],
};

const SHIFT_64K: usize = (64 as usize * 1024).trailing_zeros() as usize;
const SHIFT_512M: usize = (512 as usize * 1024 * 1024).trailing_zeros() as usize;

struct KernelVirtualLayout<const LAYOUTS: usize> {
    max_virt_addr: usize,

    layouts: [TranslationDescriptor; LAYOUTS],
}

impl<const LAYOUTS: usize> KernelVirtualLayout<LAYOUTS> {
    fn virt_addr_props(&self, virt_addr: usize) -> Result<(usize, AttributeFields), &'static str> {
        if virt_addr > self.max_virt_addr {
            return Err("Address out of range");
        }

        for i in &self.layouts {
            if (i.virtual_range)().contains(&virt_addr) {
                let output_addr = match i.map_to {
                    Some(start) => start + (virt_addr - (i.virtual_range)().start),
                    None => virt_addr,
                };

                return Ok((output_addr, i.attribute_fields));
            }
        }

        Err("Address not mapped")
    }
}

#[derive(Copy, Clone)]
pub enum MemAttributes {
    CacheableDRAM,
    Device,
}

#[derive(Copy, Clone)]
pub enum AccessPermissions {
    ReadOnly,
    ReadWrite,
}

#[derive(Copy, Clone)]
pub struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AccessPermissions,
    pub execute_never: bool,
}

pub struct TranslationDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> RangeInclusive<usize>,
    pub map_to: Option<usize>,
    pub attribute_fields: AttributeFields,
}
