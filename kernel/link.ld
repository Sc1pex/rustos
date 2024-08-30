PAGE_SIZE = 64K;
PAGE_MASK = PAGE_SIZE - 1;

RPI_PHYS_LOAD_ADDR = 0x80000;

SECTIONS
{
    . = 0;

    .boot_core (NOLOAD) : {
        . += RPI_PHYS_LOAD_ADDR;
        __boot_core_stack_end = .;
    } 

    __code_start = .;
    .text : { 
        KEEP(*(.text.boot))
        *(.text .text.*) 
    }
    .rodata : { *(.rodata .rodata.*) }
    
    . = ALIGN(PAGE_SIZE);
    __code_end = .;

    .data : { *( .data* ) }
    .bss : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        . = ALIGN(16);
        __bss_end = .;
    }
}
__bss_size = (__bss_end - __bss_start)>>3;
