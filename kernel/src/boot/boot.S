.section ".text.boot"

.global _start

_start:
    mrs x1, mpidr_el1
    and x1, x1, #3
    cbnz x1, 3f

    ldr x1, =_start
    mov sp, x1
    ldr x1, =__bss_start
    ldr w2, =__bss_size
1:  
    cbz w2, 2f
    str xzr, [x1], #8
    sub w2, w2, #1
    cbnz w2, 1b

2: 
    bl __start_rust

3:
    wfe
    b 3b
