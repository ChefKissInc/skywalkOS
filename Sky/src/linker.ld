/* Copyright (c) ChefKiss 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details. */

ENTRY(kernel_main);
OUTPUT_ARCH(x86-64);
OUTPUT_FORMAT(elf64-x86-64);

KERNEL_VIRT_OFFSET = 0xFFFFFFFF80000000;

SECTIONS {
    . = KERNEL_VIRT_OFFSET + 0x200000;
    . = ALIGN(4096);

    .text : AT(ADDR(.text) - KERNEL_VIRT_OFFSET) {
        __text_start = .;
        *(.text .text.*);
        . = ALIGN(4096);
        __text_end = .;
    }

    PROVIDE(__etext = .);

    .rodata : AT(ADDR(.rodata) - KERNEL_VIRT_OFFSET) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        . = ALIGN(4096);
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - KERNEL_VIRT_OFFSET) {
        __data_start = .;
        *(.data .data.*);
        . = ALIGN(4096);
        __data_end = .;
        __bss_start = .;
        *(.bss .bss.*);
        . = ALIGN(4096);
        __bss_end = .;
    }

    .eh_frame_hdr : AT(ADDR(.eh_frame_hdr) - KERNEL_VIRT_OFFSET) {
        __eh_frame_hdr = .;
        *(.eh_frame_hdr);
        *(.eh_frame_entry .eh_frame_entry.*);
    }

    .eh_frame : AT(ADDR(.eh_frame) - KERNEL_VIRT_OFFSET) ONLY_IF_RO {
        __eh_frame = .;
        KEEP(*(.eh_frame));
        *(.eh_frame.*);
    }

    .gcc_except_table : AT(ADDR(.gcc_except_table) - KERNEL_VIRT_OFFSET) ONLY_IF_RO {
        __gcc_except_table = .;
        *(.gcc_except_table .gcc_except_table.*);
    }

    /DISCARD/ : {
        *(.comment*)
        *(.note*)
    }
}
