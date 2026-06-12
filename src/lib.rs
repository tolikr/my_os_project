#![no_main]
#![no_std]

extern crate alloc; // Подключаем стандартный системный крейт кучи Rust

mod memory;
mod trap;
mod uart;

use core::panic::PanicInfo;
use uart::Uart;

#[global_allocator]
static ALLOCATOR: memory::heap::LinkedListAllocator = memory::heap::LinkedListAllocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hart_id: usize, fdt_address: usize) {
    Uart::print_ln("Kernel loaded.");

    Uart::print_ln("");

    // Переменные для хранения параметров RAM (значения по умолчанию на случай сбоя FDT)
    let ram_start_offset: usize = 0x0020_0000;
    let mut ram_start: usize = 0x8000_0000;
    let mut ram_size: usize = 0x0800_0000; // 128 MB

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
                    ram_start = region.starting_address as usize;
                    ram_size = region.size.unwrap_or(ram_size);
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

    // Сдвинем на оффсет если память покажет на небезопасный регион
    ram_start += ram_start_offset;
    ram_size -= ram_start_offset;

    memory::init(ram_start, ram_size);

    Uart::print_ln("Memory subsystem initialized (Pages + Heap).");

    // ПРОВЕРКА: Теперь у вас работает трансляция памяти!
    use alloc::vec::Vec;
    let mut v = Vec::new();
    v.push(hart_id);
    v.push(fdt_address);

    Uart::print_ln("Vec successfully allocated on translated memory!");

    // Инициализируем прерывания!
    trap::init_traps();

    Uart::print_ln("Traps initialized. We are bulletproof now!");

    // Проверим, ловит ли наш ассемблер ошибки?
    // Намеренно читаем из запрещенного адреса (0x0), чтобы вызвать Page Fault:
    unsafe {
        // Раскомментируем и пишем через volatile, чтобы Rust не соптимизировал это чтение
        let _trash = core::ptr::read_volatile(0x0 as *const u64);
    }
    // Если мы увидим эту строчку — значит прерывания НЕ сработали
    Uart::print_ln("Uh oh... We survived? That's bad, MMU or Traps are broken.");
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    Uart::print_ln("Panic!");
    loop {}
}
