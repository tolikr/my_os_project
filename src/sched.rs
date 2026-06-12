use crate::memory::page;
use crate::arch::trap::TrapFrame;
use crate::uart::Uart;

/// Максимальное количество потоков, которое может крутить наше мини-ядро
const MAX_TASKS: usize = 4;
/// Размер стека для каждого потока (1 страница = 4096 байт)
const STACK_SIZE: usize = 4096;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskState {
    Unused,
    Ready,
    Running,
}

/// Паспорт потока (Thread Control Block)
#[derive(Copy, Clone)]
pub struct Task {
    pub id: usize,
    pub state: TaskState,
    // Хранит текущий указатель стека (SP) потока, когда он не активен
    pub stack_pointer: usize, 
}

/// Глобальный менеджер задач
pub struct TaskManager {
    pub tasks: [Task; MAX_TASKS],
    pub current_task_id: usize,
}

// Делаем менеджер глобально доступным для ядра
pub static mut TASK_MANAGER: TaskManager = TaskManager {
    tasks: [Task { id: 0, state: TaskState::Unused, stack_pointer: 0 }; MAX_TASKS],
    current_task_id: 0,
};

impl TaskManager {
    /// Принудительная очистка и инициализация слотов
    pub fn init(&mut self) {
        for i in 0..MAX_TASKS {
            self.tasks[i] = Task {
                id: i,
                state: TaskState::Unused,
                stack_pointer: 0,
            };
        }
        self.current_task_id = 0;
    }
    
    /// Функция создания нового потока
    pub fn create_task(&mut self, entry_point: fn()) -> usize {
        // Ищем свободный слот для задачи
        for i in 0..MAX_TASKS {
            if self.tasks[i].state == TaskState::Unused {
                // 1. Выделяем физическую страницу под стек этого потока
                let stack_page = page::alloc_page();
                // Стек в RISC-V растет сверху вниз, поэтому вершина стека — это конец страницы
                let stack_top = stack_page + STACK_SIZE;

                // 2. Формируем на вершине стека ФАЛЬШИВЫЙ контекст (TrapFrame)
                // Сдвигаем указатель вниз на размер структуры TrapFrame
                let frame_size = core::mem::size_of::<TrapFrame>();
                let frame_ptr = (stack_top - frame_size) as *mut TrapFrame;

                unsafe {
                    // Обнуляем весь кадр, чтобы в регистрах не было мусора
                    core::ptr::write_bytes(frame_ptr, 0, 1);
                    
                    // Записываем «точку входа» — адрес функции, с которой начнется поток,
                    // в поле sepc. Когда ассемблер сделает sret, процессор прыгнет туда!
                    (*frame_ptr).sepc = entry_point as usize;

                    // Настраиваем sstatus: выставляем бит SPP = 1 (чтобы процессор оставался в S-mode)
                    // и бит SPIE = 1 (чтобы при старте потока прерывания/таймер были включены).
                    // В RISC-V это маска 0x120
                    (*frame_ptr).sstatus = 0x120;
                }

                // 3. Сохраняем паспорт задачи
                self.tasks[i] = Task {
                    id: i,
                    state: TaskState::Ready,
                    // Наш текущий SP для этой задачи теперь указывает на этот фальшивый кадр!
                    stack_pointer: frame_ptr as usize, 
                };

                return i;
            }
        }
        Uart::print_ln("No available slots");
        panic!("Sched: No free task slots!");
    }

    /// Алгоритм "Карусель" (Round Robin) — выбор следующей задачи
    pub fn pick_next_task(&mut self) -> usize {
        let mut next_id = self.current_task_id;
        
        for _ in 0..MAX_TASKS {
            next_id = (next_id + 1) % MAX_TASKS;
            if self.tasks[next_id].state == TaskState::Ready {
                return next_id;
            }
        }
        
        // Если готовых задач нет, возвращаем текущую (или нулевую/idle)
        self.current_task_id
    }
}