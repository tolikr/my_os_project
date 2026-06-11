use core::ops::{Deref, DerefMut};
use crate::memory::ENTRY_COUNT;

// Битовые флаги для RISC-V PTE
pub const PTE_VALID: u64 = 1 << 0;
pub const PTE_READ: u64  = 1 << 1;
pub const PTE_WRITE: u64 = 1 << 2;
pub const PTE_EXEC: u64  = 1 << 3;
pub const PTE_USER: u64  = 1 << 4;
pub const PTE_GLOBAL: u64 = 1 << 5;
pub const PTE_ACCESSED: u64 = 1 << 6;
pub const PTE_DIRTY: u64 = 1 << 7;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn new(ppn: u64, flags: u64) -> Self {
        // В Sv39 физический номер страницы (PPN) сдвигается на 10 бит влево,
        // освобождая место под флаги.
        Self((ppn << 10) | flags)
    }

    pub fn is_valid(&self) -> bool {
        (self.0 & PTE_VALID) != 0
    }

    // Если запись имеет флаги R, W или X — она указывает на реальную память (лист).
    // Если флагов нет — она указывает на следующую таблицу страниц (ветку).
    pub fn is_leaf(&self) -> bool {
        (self.0 & (PTE_READ | PTE_WRITE | PTE_EXEC)) != 0
    }

    pub fn get_ppn(&self) -> u64 {
        // Извлекаем физический адрес (номер страницы) обратно
        (self.0 >> 10) & 0x003F_FFFF_FFFF_FFFF
    }
    
    pub fn get_physical_address(&self) -> usize {
        (self.get_ppn() << 12) as usize
    }
}

// Сама таблица страниц — это просто массив из ENTRY_COUNT записей, 
// занимающий ровно одну страницу (4096 байт)
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; ENTRY_COUNT],
}