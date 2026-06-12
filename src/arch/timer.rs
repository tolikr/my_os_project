use crate::uart::Uart;

// Интервал тика таймера. 
// В QEMU частота таймера обычно 10 МГц, то есть 10_000_000 тиков — это ровно 1 секунда.
// Давайте для теста сделаем шаг в 10_000_000 тиков (1 секунда), чтобы экран не затапливало логами.
const TIMER_INTERVAL: u64 = 10_000_000;

static mut TICK_COUNT: usize = 0;

/// Функция перезаводки таймера через SBI-вызов к OpenSBI
pub fn set_next_trigger() {
    let current_time: u64;
    unsafe {
        // Читаем аппаратный счетчик времени RISC-V
        core::arch::asm!("csrr {}, time", out(reg) current_time);
    }

    let next_trigger = current_time + TIMER_INTERVAL;

    // Вызываем OpenSBI (ecall). 
    // Extension ID = 0x00 (Timer), Function ID = 0 (Set Timer)
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") 0x00,          // SBI Extension ID
            in("a6") 0,             // SBI Function ID
            in("a0") next_trigger,  // Аргумент: время следующего прерывания (младшие 64 бита)
        );
    }
}

/// Обработчик тика таймера, который мы будем вызывать из trap.rs
pub fn handle_timer_interrupt() {
    // Перезаводим будильник на следующую секунду
    set_next_trigger();

    unsafe {
        TICK_COUNT += 1;
        Uart::print_str("⏰ [Timer Tick] Number: ");
        // Печатаем номер тика в hex или просто инкремент
        Uart::print_hex(TICK_COUNT);
        Uart::print_str("\n");
    }
}

/// Включение прерываний таймера на уровне процессора
pub fn init_timer() {
    // 1. Заводим самый первый будильник
    set_next_trigger();

    unsafe {
        // 2. Разрешаем прерывания таймера в регистре sie (Supervisor Interrupt Enable)
        // Бит STIE (Supervisor Timer Interrupt Enable) — это 5-й бит (1 << 5 = 0x20)
        let sie_mask: usize = 0x20;
        core::arch::asm!("csrs sie, {}", in(reg) sie_mask);

        // 3. Включаем глобальные прерывания в регистре sstatus
        // Бит SIE (Supervisor Interrupt Enable) — это 1-й бит (1 << 1 = 0x02)
        let sstatus_mask: usize = 0x02;
        core::arch::asm!("csrs sstatus, {}", in(reg) sstatus_mask);
    }
}