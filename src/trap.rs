#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    // Все 32 общих регистра RISC-V (кроме x0, который всегда 0)
    pub regs: [usize; 32],
    // Системные регистры, которые пригодятся для отладки
    pub sstatus: usize,
    pub sepc: usize,
}

use crate::uart::Uart;

#[unsafe(no_mangle)]
pub extern "C" fn rust_trap_handler(frame: &mut TrapFrame) {
    let scause: usize;
    unsafe { core::arch::asm!("csrr {}, scause", out(reg) scause); }

    let is_interrupt = (scause >> 63) == 1;
    let code = scause & 0xfff;

    if is_interrupt {
        match code {
            1 => {
                // Прерывание программного таймера (Software Interrupt)
                // Здесь будет переключение контекста потоков (планировщик!)
            }
            5 => {
                // Аппаратный таймер (Timer Interrupt)
                // Сюда мы будем приходить каждые несколько миллисекунд
                //timer_handler();
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
        // Если это ошибка (Page Fault, Illegal Instruction)
        // Если упало само ЯДРО — то это жесткая паника (как сейчас)
        // If упала программа пользователя — мы просто убиваем эту программу, но ядро живет!
        Uart::print_str("Exception happened! Halted.\n");
        loop {}
    }
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
