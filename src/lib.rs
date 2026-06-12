#![no_main]
#![no_std]

extern crate alloc; // Подключаем стандартный системный крейт кучи Rust

mod memory;
mod uart;

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

    Uart::print_ln("Fdt id.");
    Uart::print_hex(fdt_address);
    Uart::print_ln("");

    // 2. Инициализируем парсер FDT по указателю, который нам дал QEMU
    unsafe {
        match fdt::Fdt::from_ptr(fdt_address as *const u8) {
            Ok(fdt) => {
                Uart::print_ln("FDT successfully parsed!");

                // Читаем модель платы из FDT
                Uart::print_str("Booted on board: ");
                Uart::print_ln(fdt.root().model());

                // Узнаем, сколько памяти нам РЕАЛЬНО выделил QEMU
                let memory = fdt.memory();

                Uart::print_ln("Detected RAM regions:");
                for region in memory.regions() {
                    Uart::print_str("  - Start: ");
                    Uart::print_hex(region.starting_address as usize);
                    Uart::print_str("    Size: ");
                    Uart::print_hex(region.size.unwrap_or(0) as usize);
                }

                // Посчитаем количество ядер процессора (Harts)
                let cpus_count = fdt.cpus().count();
                Uart::print_ln("");
                Uart::print_ln("Number of CPU cores: ");
                // (Для вывода чисел можно использовать Uart::print_hex или перевести в строку)
                Uart::print_hex(cpus_count);
            }
            Err(_) => {
                Uart::print_ln("ERROR: Failed to parse FDT header!");
            }
        }
    }

    Uart::print_ln("");
    Uart::print_ln("--------------------------");
    
    memory::init();

    // ПРОВЕРКА: Теперь у вас работает трансляция памяти!
    let mut v = Vec::new();
    v.push(hart_id);
    v.push(fdt_address);

    Uart::print_ln("Vec successfully allocated on translated memory!");
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    Uart::print_ln("Panic!");
    loop {}
}
