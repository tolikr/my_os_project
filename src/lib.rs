#![no_main]
#![no_std]

extern crate alloc; // Подключаем стандартный системный крейт кучи Rust

mod uart;
mod memory;

use core::panic::PanicInfo;
use uart::Uart;

#[global_allocator]
static ALLOCATOR: memory::heap::LinkedListAllocator = memory::heap::LinkedListAllocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hart_id: usize, fdt_address: usize) {

    Uart::print_ln("Kernel loaded.");

    memory::init();
    
    Uart::print_ln("Memory subsystem fully initialized (Pages + Heap).");

    // ПРОВЕРКА: Теперь у вас работает магия Rust!
    use alloc::vec::Vec;
    let mut v = Vec::new();
    v.push(hart_id);
    v.push(fdt_address);
    
    Uart::print_ln("Vec successfully allocated on RISC-V 64 heap!");

    Uart::print_str("--------------------------\n");


    Uart::print_ln("Hart id.");
    Uart::print_hex(hart_id);

    Uart::print_hex(fdt_address);

    
    Uart::print_str("--------------------------\n");
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    Uart::print_ln("Panic!");
    loop {}
}
