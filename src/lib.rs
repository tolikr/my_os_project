#![no_main]
#![no_std]

extern crate alloc; // Подключаем стандартный системный крейт кучи Rust

mod arch;
mod memory;
mod sched;
mod uart;

use core::panic::PanicInfo;
use uart::Uart;

use crate::sched::TASK_MANAGER;

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
    arch::trap::init_traps();
    Uart::print_ln("Traps initialized. We are bulletproof now!");

    // 2. Создаем две независимые задачи
    unsafe {
        // Получаем безопасную ссылку на менеджер задач через сырой указатель
        let s: &mut sched::TaskManager = &mut *core::ptr::addr_of_mut!(TASK_MANAGER);

        s.init();

        let id_a = s.create_task(task_alpha);
        let id_b = s.create_task(task_beta);

        // Переводим первую задачу в состояние Running, чтобы ядро знало, с кого начать
        s.tasks[id_a].state = crate::sched::TaskState::Running;
        s.current_task_id = id_a;
    }
    Uart::print_ln("OS Tasks created and scheduled.");

    // 2. Заводим и активируем системный таймер!
    arch::timer::init_timer();
    Uart::print_ln("Timer initialized. Core heartbeat started.");

    // после выполнения этого кода система уже будет работать сама по прерываниям от процессора
    unsafe {
        let s = &*core::ptr::addr_of!(TASK_MANAGER);
        let initial_sp = s.tasks[s.current_task_id].stack_pointer;

        core::arch::asm!(
            "mv sp, {}",   // Загружаем SP нашей первой задачи (Task Alpha)
            "j trap_return", // Прыгаем на глобальную метку в trap.S!
            in(reg) initial_sp
        );
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    Uart::print_ln("Panic!");
    loop {}
}

// тестируем задачи
fn task_alpha() {
    loop {
        Uart::print_ln("🅰️  [Task Alpha] Hello from first thread!");
        // Немного подождем в цикле, чтобы не заспамить UART слишком быстро
        for _ in 0..5_000_000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
}

fn task_beta() {
    loop {
        Uart::print_ln("🅱️  [Task Beta] Greetings from second thread!");
        for _ in 0..5_000_000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
}
