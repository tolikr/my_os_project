use core::u8;

pub struct Uart;

impl Uart {
    // Вывод обычной строки
    pub fn print_str(s: &str) {
        for b in s.bytes() {
            putchar(b);
        }
    }

    
    pub fn print_ln(s: &str) {
        for b in s.bytes() {
            putchar(b);
        }
        putchar(b'\n');
    }

    // Вывод числа в шестнадцатеричном формате (очень нужно для отладки адресов)
    pub fn print_hex(mut num: usize) {
        Self::print_str("0x");
        if num == 0 {
            putchar(b'0');
            return;
        }
        let mut buf = [0u8; 16];
        let mut i = 0;
        while num > 0 {
            let nibble = (num & 0xf) as u8;
            buf[i] = if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) };
            num >>= 4;
            i += 1;
        }
        for idx in (0..i).rev() {
            putchar(buf[idx]);
        }
    }
}

// адрес порта для вывода
const UART: *mut u8 = 0x0900_0000 as *mut u8;

fn putchar(c: u8) {
    unsafe {
        *UART = c;
    }
}
