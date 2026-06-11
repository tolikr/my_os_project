// Наша новая красивая константа размера страницы
pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = PAGE_SIZE / core::mem::size_of::<u64>();

// Импортируем метку конца ядра из линкера
unsafe extern "C" {
    static _end: u8;
}

// Храним адрес первой свободной страницы памяти
static mut FREE_MEM_START: usize = 0;

/// Инициализирует указатель на свободную память сразу за концом ядра
pub fn init_page_allocator() {
    unsafe {
        // Стартуем прямо за концом программы
        FREE_MEM_START = &_end as *const u8 as usize;
    }
}

// Функция выделения ОДНОЙ физической страницы (PAGE_SIZE байт)
pub fn alloc_page() -> usize {
    unsafe {
        let page_address = FREE_MEM_START;
        // Сдвигаем указатель на PAGE_SIZE Байт вперед для следующего вызова
        FREE_MEM_START += PAGE_SIZE; 
        
        // Обязательно очищаем страницу нулями, чтобы там не было мусора
        let ptr = page_address as *mut u64;
        for i in 0..ENTRY_COUNT {
            ptr.add(i).write_volatile(0);
        }
        
        page_address // Возвращаем физический адрес выделенной страницы
    }
}

/// Функция для отладки: позволяет узнать текущий адрес свободной памяти
pub fn get_free_mem_start() -> usize {
    unsafe { FREE_MEM_START }
}