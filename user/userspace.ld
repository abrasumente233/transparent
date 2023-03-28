OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x40000000;

SECTIONS {
    . = BASE_ADDRESS;

    .text :   { *(.text.startup .text .text.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data :   { *(.data .data.*) }
    .bss :    {
        *(.bss)
        *(.bss.*)
        *(.sbss.*)
    }
    
    /DISCARD/ : {
        *(.debug_*)
    }
}