#![no_main]
#![no_std]

use core::panic::PanicInfo;

const UART: *mut u8 = 0x0900_0000 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    print(b"Hello, from Rust!\n");
}

fn putchar(c: u8) {
    unsafe {
        *UART = c;
    }
}

fn print(s: &[u8]) {
    for &c in s {
        putchar(c);
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    print(b"Panic!\n");
    loop {}
}