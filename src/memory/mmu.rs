use core::ops::{Deref, DerefMut};
use crate::memory::ENTRY_COUNT;
use crate::memory::PAGE_SIZE;

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

impl PageTable {
    /// Находит или создает запись (PTE) для конкретного виртуального адреса на уровне L0
    pub fn walk_mut(&mut self, va: usize) -> Option<&mut PageTableEntry> {
        // --- УРОВЕНЬ 2 (Корень) ---
        let idx2 = lv2_index(va);
        let pte2 = &mut self.entries[idx2];
        
        let mut table1_addr = if pte2.is_valid() {
            pte2.get_physical_address()
        } else {
            // Если таблицы L1 еще нет — выделяем её через наш постраничный аллокатор!
            let new_page = crate::memory::page::alloc_page();
            if new_page == 0 { return None; } // На случай, если память совсем кончилась
            
            // Связываем L2 с новой таблицей L1 (флаг VALID, но без R/W/X, так как это ветка)
            let ppn = (new_page >> 12) as u64;
            *pte2 = PageTableEntry::new(ppn, PTE_VALID);
            new_page
        };

        // --- УРОВЕНЬ 1 (Середина) ---
        let table1 = unsafe { &mut *(table1_addr as *mut PageTable) };
        let idx1 = lv1_index(va);
        let pte1 = &mut table1.entries[idx1];

        let table0_addr = if pte1.is_valid() {
            pte1.get_physical_address()
        } else {
            // Если таблицы L0 еще нет — выделяем её
            let new_page = crate::memory::page::alloc_page();
            if new_page == 0 { return None; }
            
            let ppn = (new_page >> 12) as u64;
            *pte1 = PageTableEntry::new(ppn, PTE_VALID);
            new_page
        };

        // --- УРОВЕНЬ 0 (Лист) ---
        let table0 = unsafe { &mut *(table0_addr as *mut PageTable) };
        let idx0 = lv0_index(va);
        
        // Возвращаем прямую ссылку на финальную запись, которую мы теперь можем модифицировать
        Some(&mut table0.entries[idx0])
    }

    /// Отображает виртуальную страницу на физическую с заданными флагами
    pub fn map_page(&mut self, va: usize, pa: usize, flags: u64) -> Result<(), &'static str> {
        // Проверяем, что адреса ровно выровнены по границе страницы
        if (va % PAGE_SIZE) != 0 || (pa % PAGE_SIZE) != 0 {
            return Err("Addresses must be page-aligned!");
        }

        // Ищем финальную запись в таблице L0
        if let Some(pte) = self.walk_mut(va) {
            if pte.is_valid() {
                return Err("Virtual address is already mapped!");
            }
            
            // Прописываем физический адрес (PPN) и флаги (обязательно добавляем PTE_VALID)
            let ppn = (pa >> 12) as u64;
            *pte = PageTableEntry::new(ppn, flags | PTE_VALID);
            Ok(())
        } else {
            Err("Out of memory while allocating page tables!")
        }
    }
}

/// Извлекает индекс для корневой таблицы L2 (биты 38-30)
fn lv2_index(va: usize) -> usize {
    (va >> 30) & 0x1FF
}

/// Извлекает индекс для промежуточной таблицы L1 (биты 29-21)
fn lv1_index(va: usize) -> usize {
    (va >> 21) & 0x1FF
}

/// Извлекает индекс для финальной таблицы L0 (биты 20-12)
fn lv0_index(va: usize) -> usize {
    (va >> 12) & 0x1FF
}