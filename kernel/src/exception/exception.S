.macro CALL_WITH_CONTEXT handler
__vector_\handler:
    // Make room for registers
    sub sp, sp, #16 * 17

    stp	x0,  x1,  [sp, #16 * 0]
	stp	x2,  x3,  [sp, #16 * 1]
	stp	x4,  x5,  [sp, #16 * 2]
	stp	x6,  x7,  [sp, #16 * 3]
	stp	x8,  x9,  [sp, #16 * 4]
	stp	x10, x11, [sp, #16 * 5]
	stp	x12, x13, [sp, #16 * 6]
	stp	x14, x15, [sp, #16 * 7]
	stp	x16, x17, [sp, #16 * 8]
	stp	x18, x19, [sp, #16 * 9]
	stp	x20, x21, [sp, #16 * 10]
	stp	x22, x23, [sp, #16 * 11]
	stp	x24, x25, [sp, #16 * 12]
	stp	x26, x27, [sp, #16 * 13]
	stp	x28, x29, [sp, #16 * 14] 

    mrs x1, ELR_EL1
    mrs x2, SPSR_EL1
    mrs x3, ESR_EL1

    stp lr, x1, [sp, #16 * 15]
    stp x2, x3, [sp, #16 * 16]

    // x0 is the first argument for a function
    mov x0, sp
    bl \handler

    b __exception_restore_context

.endm

.section .text
.align 11

.global __exception_vector_start
__exception_vector_start:

// Exceptions from current EL while using SP_EL0
.org 0x000
    CALL_WITH_CONTEXT current_el0_sync
.org 0x080
    CALL_WITH_CONTEXT current_el0_irq
.org 0x100
    CALL_WITH_CONTEXT current_el0_fiq
.org 0x180
    CALL_WITH_CONTEXT current_el0_serror

// Exceptions from cuurrent EL while using SP_ELx, x != 0
.org 0x200
    CALL_WITH_CONTEXT current_elx_sync
.org 0x280
    CALL_WITH_CONTEXT current_elx_irq
.org 0x300
    CALL_WITH_CONTEXT current_elx_fiq
.org 0x380
    CALL_WITH_CONTEXT current_elx_serror

// Exceptions from a lowe EL, AArch64
.org 0x400
    CALL_WITH_CONTEXT lower_el_aarch64_sync
.org 0x480
    CALL_WITH_CONTEXT lower_el_aarch64_irq
.org 0x500
    CALL_WITH_CONTEXT lower_el_aarch64_fiq
.org 0x580
    CALL_WITH_CONTEXT lower_el_aarch64_serror

// Exceptions from a lowe EL, AArch32
// These are probably impossible
.org 0x600
    CALL_WITH_CONTEXT lower_el_aarch32_sync
.org 0x680
    CALL_WITH_CONTEXT lower_el_aarch32_irq
.org 0x700
    CALL_WITH_CONTEXT lower_el_aarch32_fiq
.org 0x780
    CALL_WITH_CONTEXT lower_el_aarch32_serror

__exception_restore_context:
	ldr	w19,      [sp, #16 * 16]
	ldp	lr,  x20, [sp, #16 * 15]

	msr	SPSR_EL1, x19
	msr	ELR_EL1,  x20

	ldp	x0,  x1,  [sp, #16 * 0]
	ldp	x2,  x3,  [sp, #16 * 1]
	ldp	x4,  x5,  [sp, #16 * 2]
	ldp	x6,  x7,  [sp, #16 * 3]
	ldp	x8,  x9,  [sp, #16 * 4]
	ldp	x10, x11, [sp, #16 * 5]
	ldp	x12, x13, [sp, #16 * 6]
	ldp	x14, x15, [sp, #16 * 7]
	ldp	x16, x17, [sp, #16 * 8]
	ldp	x18, x19, [sp, #16 * 9]
	ldp	x20, x21, [sp, #16 * 10]
	ldp	x22, x23, [sp, #16 * 11]
	ldp	x24, x25, [sp, #16 * 12]
	ldp	x26, x27, [sp, #16 * 13]
	ldp	x28, x29, [sp, #16 * 14]

	add	sp,  sp,  #16 * 17

	eret

