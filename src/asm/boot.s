.option norvc

.section .text.entry
.global _start
_start:
    la sp, boot_stack_top

    tail main

.section .bss.bootstack
boot_stack:
.global boot_stack
.space 4096 * 8 # 2^15 bytes
boot_stack_top:
.global boot_stack_top
