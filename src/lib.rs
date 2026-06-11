#![no_main]
#![no_std]

mod uart;

use core::panic::PanicInfo;
use uart::Uart;

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hart_id: usize, fdt_address: usize) {

    Uart::print_ln("Kernel loaded!");
    
    Uart::print_hex(hart_id);

    Uart::print_hex(fdt_address);

    
    Uart::print_str("--------------------------\n");
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    Uart::print_ln("Panic!");
    loop {}
}
