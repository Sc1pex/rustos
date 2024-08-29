#[macro_export]
macro_rules! read_reg {
    ($name: expr) => {{
        let x: u64;
        unsafe { ::core::arch::asm!(concat!("mrs {}, ", $name), out(reg) x) }
        x
    }};
}

#[macro_export]
macro_rules! write_reg {
    ($name: expr, $value: expr) => {{
        unsafe { ::core::arch::asm!(concat!("msr ", $name, ", {}"), in(reg) ($value as u64)) }
    }};
}
