/* Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details. */

ENTRY(kernel_main)
OUTPUT_ARCH(x86-64)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_VIRT_OFFSET = 0xFFFFFFFF80000000;

SECTIONS {
    . = KERNEL_VIRT_OFFSET + 0x200000;
    . = ALIGN(4096);

    .text : AT(ADDR(.text) - KERNEL_VIRT_OFFSET) {
        __text_start = .;
        *(.text*)
        . = ALIGN(4096);
        __text_end = .;
    }

    PROVIDE(__etext = .);

    .rodata : AT(ADDR(.rodata) - KERNEL_VIRT_OFFSET) {
        __rodata_start = .;
        *(.rodata*)
        . = ALIGN(4096);
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - KERNEL_VIRT_OFFSET) {
        __data_start = .;
        *(.data*)
        . = ALIGN(4096);
        __data_end = .;
        __bss_start = .;
        *(.bss*)
        . = ALIGN(4096);
        __bss_end = .;
    }

    .eh_frame : AT(ADDR(.eh_frame) - KERNEL_VIRT_OFFSET) {
        __eh_frame = .;
        KEEP(*(.eh_frame*));
    }

    /DISCARD/ : {
        *(.comment*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }
}
