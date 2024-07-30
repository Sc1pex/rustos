use core::arch::asm;

#[derive(Debug)]
pub enum PrivilegeLevel {
    Application,
    Kernel,
    Hypervisor,
    Unknown,
}

pub fn current_el() -> PrivilegeLevel {
    let el = unsafe {
        let x: u64;
        asm!("mrs {}, CurrentEL", out(reg) x);
        x
    };
    match el {
        0b0000 => PrivilegeLevel::Application,
        0b0100 => PrivilegeLevel::Kernel,
        0b1000 => PrivilegeLevel::Hypervisor,
        _ => PrivilegeLevel::Unknown,
    }
}
