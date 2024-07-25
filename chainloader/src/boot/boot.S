.section ".text.boot"

.global _start

// Load the address of a symbol into a register, absolute.
//
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_ABS register, symbol
	movz	\register, #:abs_g2:\symbol
	movk	\register, #:abs_g1_nc:\symbol
	movk	\register, #:abs_g0_nc:\symbol
.endm

_start:
    mrs x1, mpidr_el1
    and x1, x1, #3
    cbnz x1, 3f

    ldr x1, =__bss_start
    ldr w2, =__bss_size
1:  
    cbz w2, 2f
    str xzr, [x1], #8
    sub w2, w2, #1
    cbnz w2, 1b

2: 
    // Kernel is loaded at __load_addr, but is linked to be loaded at __bin_start
    // So move it there, making space for a new kernel to be sent over serial
    ldr x1, =__load_addr 
    ldr x2, =__bin_start
    ldr x3, =__bin_end

4: 
    ldr x4, [x1], #8
    str x4, [x2], #8
    cmp x2, x3
    bne 4b

    ldr x1, =_start
    mov sp, x1

    // Now the kernel is at __bin_start, where the linker assumed it would be
    ADR_ABS x1, __start_rust
    br x1

3:
    wfe
    b 3b

