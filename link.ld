SECTIONS
{
    . = 0x80000;
    .text : { 
        KEEP(*(.text.boot))
        *(.text .text.* .gnu.linkonce.t*) 
    }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }
}
__bss_size = (__bss_end - __bss_start)>>3;
