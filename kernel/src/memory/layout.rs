use core::range::RangeInclusive;

#[derive(Copy, Clone)]
pub(super) enum MemAttributes {
    CacheableDRAM,
    Device,
}

#[derive(Copy, Clone)]
pub(super) enum AccessPermissions {
    ReadOnly,
    ReadWrite,
}

#[derive(Copy, Clone)]
pub(super) struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AccessPermissions,
    pub execute_never: bool,
}

struct TranslationDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> RangeInclusive<usize>,
    pub map_to: Option<usize>,
    pub attribute_fields: AttributeFields,
}

pub(super) struct KernelVirtualLayout<const LAYOUTS: usize> {
    max_virt_addr: usize,

    layouts: [TranslationDescriptor; LAYOUTS],
}

impl<const LAYOUTS: usize> KernelVirtualLayout<LAYOUTS> {
    pub fn virt_addr_props(
        &self,
        virt_addr: usize,
    ) -> Result<(usize, AttributeFields), &'static str> {
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

pub(super) static KERNEL_LAYOUT: KernelVirtualLayout<4> = KernelVirtualLayout {
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

pub fn print_kernel_memory_layout() {
    crate::info!("MMU Translations:");
    for t in &KERNEL_LAYOUT.layouts {
        crate::info!("    {t}");
    }
}

impl core::fmt::Display for TranslationDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let start = (self.virtual_range)().start;
        let end = (self.virtual_range)().end;
        let size = end - start + 1;

        const KIB: usize = 1024;
        const MIB: usize = 1024 * 1024;
        const GIB: usize = 1024 * 1024 * 1024;

        let (size, unit) = if (size / GIB) > 0 {
            (size.div_ceil(GIB), "GiB")
        } else if (size / MIB) > 0 {
            (size.div_ceil(MIB), "MiB")
        } else if (size / KIB) > 0 {
            (size.div_ceil(KIB), "KiB")
        } else {
            (size, "Byte")
        };

        let attr = match self.attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => "RAM",
            MemAttributes::Device => "Dev",
        };

        let access = match self.attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => "RO",
            AccessPermissions::ReadWrite => "RW",
        };

        let execute = if self.attribute_fields.execute_never {
            "PXN"
        } else {
            "PX"
        };

        write!(
            f,
            "{:28}: {:#010X} - {:#010X} | {:3} {} | {} {} {}",
            self.name, start, end, size, unit, attr, access, execute
        )
    }
}
