#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    // Все 32 общих регистра RISC-V (кроме x0, который всегда 0)
    pub regs: [usize; 32],
    // Системные регистры, которые пригодятся для отладки
    pub sstatus: usize,
    pub sepc: usize,
}

use crate::arch::timer;
use crate::sched::TASK_MANAGER;
use crate::uart::Uart; // Подключаем наш менеджер задач

#[unsafe(no_mangle)]
pub extern "C" fn rust_trap_handler(frame: &mut TrapFrame, current_sp: usize) -> usize {
    let scause: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) scause);
    }

    let is_interrupt = (scause >> 63) == 1;
    let code = scause & 0xfff;
    // По умолчанию мы возвращаем тот же стек, что и пришел (никуда не переключаемся)
    let mut next_sp = current_sp;

    if is_interrupt {
        match code {
            1 => {
                // Прерывание программного таймера (Software Interrupt)
                // Здесь будет переключение контекста потоков (планировщик!)
            }
            5 => {
                // Аппаратный таймер (Timer Interrupt)
                // Сюда мы будем приходить каждые несколько миллисекунд
                timer::handle_timer_interrupt();
                // 2. Логика переключения контекста
                unsafe {
                    let s = unsafe { &mut *core::ptr::addr_of_mut!(TASK_MANAGER) };
                    
                    // Сохраняем указатель стека у текущей задачи
                    let current_id = s.current_task_id;
                    s.tasks[current_id].stack_pointer = current_sp;
                    
                    // Если текущая задача работала, переводим её обратно в статус Ready
                    if s.tasks[current_id].state == crate::sched::TaskState::Running {
                        s.tasks[current_id].state = crate::sched::TaskState::Ready;
                    }

                    // Выбираем следующую задачу из очереди
                    let next_id = s.pick_next_task();
                    s.current_task_id = next_id;
                    s.tasks[next_id].state = crate::sched::TaskState::Running;

                    // Возвращаем указатель стека НОВОЙ задачи в ассемблер!
                    next_sp = s.tasks[next_id].stack_pointer;
                }
            }
            9 => {
                // Внешнее прерывание (External Interrupt - UART, Диск и т.д.)
                //external_handler();
            }
            _ => {
                Uart::print_str("Unknown interrupt\n");
            }
        }
    } else {
        // Если это критическая ошибка (как наш прошлый Page Fault) — зависаем
        Uart::print_str("\n🎄 [OS Exception] 🎄\n");
        Uart::print_str("Reason Code: ");
        Uart::print_hex(code);
        Uart::print_str("\nInstruction Pointer (sepc): ");
        Uart::print_hex(frame.sepc);
        Uart::print_ln("\nSystem halted.");
        loop {}
    }
    
    // Ассемблер получит этот адрес через регистр a0
    next_sp
}

/// Функция для регистрации нашего вектора в процессоре
pub fn init_traps() {
    unsafe extern "C" {
        // Кнопка подхвата из ассемблера
        fn trap_vector();
    }

    unsafe {
        // Записываем адрес asm-функции trap_vector в системный регистр stvec
        core::arch::asm!("csrw stvec, {}", in(reg) trap_vector as usize);
    }
}
