.section .text
.global _start

_start:
    ldr x30, =stack_top
    mov sp, x30
    // Явно загружаем адрес, где QEMU оставил наше дерево устройств, в регистр x0. 
    // Первый аргумент нижевызываемого метода.
    ldr     x0, =0x40000000
    // Запустим ядро
    bl kmain
    b .