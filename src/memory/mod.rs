pub mod page;
pub mod heap;
pub mod mmu;

// Выносим константу размера страницы на уровень выше для удобства
pub use page::PAGE_SIZE;
pub use page::ENTRY_COUNT;

// Сюда же можно перенести глобальную константу размера кучи
pub const HEAP_PAGE_COUNT: usize = 256; 

/// Высокоуровневая функция инициализации ВСЕЙ памяти ядра
pub fn init() {
    // 1. Запускаем постраничный аллокатор
    page::init_page_allocator();
    
    // 2. Выделяем страницы под кучу Rust
    let heap_start = page::alloc_page();
    for _ in 0..(HEAP_PAGE_COUNT - 1) {
        page::alloc_page();
    }
    let heap_size = HEAP_PAGE_COUNT * page::PAGE_SIZE;

    // 3. Передаем эту память в LinkedListAllocator
    unsafe {
        crate::ALLOCATOR.init(heap_start, heap_size);
    }
    
}

pub fn init_mmu(ram_start: usize, ram_size: usize) {
    // 3. НОВИНКА: Запускаем MMU и переходим в виртуальную память!
    mmu::init_mmu(ram_start, ram_size);
}