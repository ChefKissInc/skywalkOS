/*
    Copyright (c) ChefKiss Inc 2021-2022.
    This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.
*/

ENTRY(kernel_main)
OUTPUT_ARCH(x86-64)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_VIRT_OFF = 0xFFFFFFFF80000000;

SECTIONS
{
    . = KERNEL_VIRT_OFF + 0x200000;
    . = ALIGN(4096);

    .text ALIGN(4096) : AT(ADDR(.text) - KERNEL_VIRT_OFF)
    {
        __text_start = .;
        *(.text*)
        . = ALIGN(4096);
        __text_end = .;
    }

    PROVIDE(__etext = .);

    .rodata ALIGN(4096) : AT(ADDR(.rodata) - KERNEL_VIRT_OFF)
    {
        __rodata_start = .;
        *(.rodata*)
        . = ALIGN(4096);
        __rodata_end = .;
    }

    .data ALIGN(4096) : AT(ADDR(.data) - KERNEL_VIRT_OFF)
    {
        __data_start = .;
        *(.data*)
        . = ALIGN(4096);
        __data_end = .;
        __bss_start = .;
        *(.bss*)
        . = ALIGN(4096);
        __bss_end = .;
    }


    .tdata ALIGN(4096) : AT(ADDR(.tdata) - KERNEL_VIRT_OFF) {
        __tdata_start = .;
        *(.tdata*)
        . = ALIGN(4096);
        __tdata_end = .;
        __tbss_start = .;
        *(.tbss*)
        . += 8;
        . = ALIGN(4096);
        __tbss_end = .;
    }

    .eh_frame ALIGN(4096) : AT(ADDR(.eh_frame) - KERNEL_VIRT_OFF)
    {
        __eh_frame = .;
        KEEP(*(.eh_frame));
    }

    __kernel_top = .;
}