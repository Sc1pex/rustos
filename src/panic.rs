use crate::driver::uart::UARTDriver;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let uart = UARTDriver::new();
    let _ = write!(uart, "KERNEL PANIC!: {}\n", info.message());
    if let Some(loc) = info.location() {
        let _ = write!(uart, "in file {}:{}", loc.file(), loc.line());
    }

    loop {}
}
