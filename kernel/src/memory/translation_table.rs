use super::layout::*;

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
pub(super) struct TranslationTables<const TABLES: usize> {
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

    pub fn populate(&mut self) -> Result<(), &'static str> {
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

    pub fn phys_base_addr(&self) -> u64 {
        let s = &self.lvl2;
        s as *const _ as u64
    }
}

const KERNEL_LV2_TABLES: usize = (super::map::END_INCLUSIVE + 1) >> SHIFT_512M;
pub(super) static mut KERNEL_TABLES: TranslationTables<KERNEL_LV2_TABLES> =
    TranslationTables::new();

const SHIFT_64K: usize = (64 as usize * 1024).trailing_zeros() as usize;
const SHIFT_512M: usize = (512 as usize * 1024 * 1024).trailing_zeros() as usize;
