#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod uart;

use uart::Uart;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    Uart::print_ln("Hello, from Rust!");
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    Uart::print_ln("Panic!");
    loop {}
}