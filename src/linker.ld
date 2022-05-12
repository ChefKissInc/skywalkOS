/*
    Copyright (c) VisualDevelopment 2021-2022.
    This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
*/

ENTRY(kernel_main)
OUTPUT_ARCH(x86-64)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_VIRT_OFF = 0xFFFFFFFF80000000;

SECTIONS
{
    PROVIDE(__executable_start = KERNEL_VIRT_OFF + 0x200000);
    . = KERNEL_VIRT_OFF + 0x200000 + SIZEOF_HEADERS;
    . = ALIGN(4K);

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VIRT_OFF)
    {
        *(.text*)
    }

    PROVIDE(__etext = .);

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_VIRT_OFF)
    {
        *(.rodata*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_VIRT_OFF)
    {
        *(.data*)
    }

    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_VIRT_OFF)
    {
        __bss_start = .;
        *(COMMON)
        *(.bss*)
        __bss_end = .;
    }

    .eh_frame ALIGN(4K) : AT(ADDR(.eh_frame) - KERNEL_VIRT_OFF)
    {
        __eh_frame = .;
        KEEP(*(.eh_frame))
        *(.eh_frame.*)
    }

    __kernel_top = .;
}