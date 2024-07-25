__load_addr = 0x80000;

ENTRY(__load_addr)

SECTIONS
{
    . = 0x2000000;
    __bin_start = .;
    .text : { 
        KEEP(*(.text.boot))
        *(.text .text.*) 
    }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    . = ALIGN(8);
    __bin_end = .;

    .bss : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        . = ALIGN(16);
        __bss_end = .;
    }
}
__bss_size = (__bss_end - __bss_start)>>3;
