OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS {
    . = BASE_ADDRESS;
    
    PROVIDE(_kernel_start = .);
    
    PROVIDE(_text_start = .);
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    PROVIDE(_text_end = .);
    
    . = ALIGN(4K);
    PROVIDE(_rodata_start = .);
    .rodata : {
        *(.rodata .rodata.*)
    }
    PROVIDE(_rodata_end = .);

    . = ALIGN(4K);
    PROVIDE(_data_start = .);
    .data : {
        *(.data .data.*)
    }
    PROVIDE(_data_end = .);
    
    . = ALIGN(4K);
    .bss : {
        PROVIDE(_bss_bootstack_start = .);
        *(.bss.bootstack)
        PROVIDE(_bss_start = .);
        *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    }

    PROVIDE(_kernel_end = .);

    /DISCARD/ : {
        *(.eh_frame)
    }
}