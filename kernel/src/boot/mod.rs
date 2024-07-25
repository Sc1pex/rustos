use crate::kernel::kernel_init;
use core::arch::global_asm;

global_asm!(include_str!("boot.S"));

#[no_mangle]
pub unsafe fn __start_rust() -> ! {
    kernel_init()
}
