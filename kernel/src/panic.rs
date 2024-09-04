use crate::{fatal, println};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    fatal!("KERNEL PANIC!:\n{}", info.message());

    if let Some(loc) = info.location() {
        println!("in file {}:{}:{}", loc.file(), loc.line(), loc.column())
    }

    loop {}
}
