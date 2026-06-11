#![no_main]
#![no_std]

mod uart;

use core::panic::PanicInfo;
use uart::Uart;

// Импортируем метку из линкера
unsafe extern "C" {
    static _end: u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hart_id: usize, fdt_address: usize) {

    Uart::print_ln("Kernel loaded.");

    init_page_allocator();
    Uart::print_ln("Memory allocator initialized.");
    unsafe { Uart::print_hex(FREE_MEM_START);}
    
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

// Храним адрес первой свободной страницы памяти
static mut FREE_MEM_START: usize = 0;

pub fn init_page_allocator() {
    unsafe {
        // Стартуем прямо за концом программы
        FREE_MEM_START = &_end as *const u8 as usize;
    }
}

// Функция выделения ОДНОЙ физической страницы (4096 байт)
pub fn alloc_page() -> usize {
    unsafe {
        let page_address = FREE_MEM_START;
        // Сдвигаем указатель на 4 КБ вперед для следующего вызова
        FREE_MEM_START += 4096; 
        
        // Обязательно очищаем страницу нулями, чтобы там не было мусора
        let ptr = page_address as *mut u64;
        for i in 0..512 {
            ptr.add(i).write_volatile(0);
        }
        
        page_address // Возвращаем физический адрес выделенной страницы
    }
}