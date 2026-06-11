use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

// Структура «узла» в нашей свободной памяти
struct FreeNode {
    size: usize,
    next: Option<&'static mut FreeNode>,
}

pub struct LinkedListAllocator {
    // Используем UnsafeCell, чтобы легально менять список внутри GlobalAlloc
    head: UnsafeCell<FreeNode>,
}

// Заявляем, что наш аллокатор безопасен для использования в глобальном контексте
unsafe impl Sync for LinkedListAllocator {}

impl LinkedListAllocator {
    // Создаем пустой аллокатор
    pub const fn new() -> Self {
        Self {
            head: UnsafeCell::new(FreeNode {
                size: 0,
                next: None,
            }),
        }
    }

    // Инициализируем аллокатор конкретным регионом физической памяти RISC-V
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        // Проверяем базовое выравнивание самого начала кучи
        let aligned_start = (heap_start + 7) & !7;
        let aligned_size = heap_size - (aligned_start - heap_start);

        let raw_node = aligned_start as *mut FreeNode;
        ptr::write(raw_node, FreeNode {
            size: aligned_size,
            next: None,
        });

        // Записываем этот первый гигантский блок в голову списка
        let head = &mut *self.head.get();
        head.next = Some(&mut *raw_node);
    }
}

// Вспомогательная функция для округления адреса вверх под требования выравнивания
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let mut current = &mut *self.head.get();

        // Ищем первый подходящий свободный блок (First Fit)
        while let Some(ref mut node) = current.next {
            let node_addr = *node as *mut FreeNode as usize;
            
            // Вычисляем, где начнется полезная память с учетом выравнивания
            let alloc_start = align_up(node_addr, align);
            let alloc_end = alloc_start + size;
            let block_end = node_addr + node.size;

            // Помещается ли запрашиваемый размер в текущий свободный блок?
            if alloc_end <= block_end {
                // Забираем узел из списка
                let mut taken_node = current.next.take().unwrap();
                
                // Если после отрезания куска в конце блока осталось место, 
                // возвращаем остаток обратно в список свободных блоков
                if block_end - alloc_end >= core::mem::size_of::<FreeNode>() {
                    let rem_node_ptr = alloc_end as *mut FreeNode;
                    ptr::write(rem_node_ptr, FreeNode {
                        size: block_end - alloc_end,
                        next: taken_node.next.take(),
                    });
                    current.next = Some(&mut *rem_node_ptr);
                } else {
                    current.next = taken_node.next.take();
                }

                return alloc_start as *mut u8;
            }
            // Идем дальше по списку
            current = current.next.as_mut().unwrap();
        }

        // Если свободного места не нашли — возвращаем Null (Out of Memory)
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // При освобождении памяти мы просто берем этот кусок,
        // создаем в нем FreeNode и пихаем в самое начало списка (в голову head)
        let node_ptr = ptr as *mut FreeNode;
        
        let head = &mut *self.head.get();
        
        ptr::write(node_ptr, FreeNode {
            size: layout.size(),
            next: head.next.take(),
        });
        
        head.next = Some(&mut *node_ptr);
    }
}