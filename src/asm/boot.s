.section .text.entry
.global _start

_start:
    la sp, stack_top

    call kmain

1:  j 1b