ENTRY(kernel_main)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_VMA = 0xFFFFFFFF80000000;
KERNEL_LMA = 0x200000;

SECTIONS {
    PROVIDE(__executable_start = KERNEL_VMA + KERNEL_LMA);
    . = KERNEL_VMA + KERNEL_LMA + SIZEOF_HEADERS;

    .kaboom ALIGN(0x1000) : AT(ADDR(.kaboom) - KERNEL_VMA) { *(.kaboom) }

    .text ALIGN(0x1000) : AT(ADDR(.text) - KERNEL_VMA) { *(.text*) }

    .rodata ALIGN(0x1000) : AT(ADDR(.rodata) - KERNEL_VMA) { *(.rodata.*) }

    .eh_frame_hdr ALIGN(0x1000) : AT(ADDR(.eh_frame) - KERNEL_VMA) {
        __eh_frame_hdr_start = .;
        *(.eh_frame*)
        __eh_frame_hdr_end = .;
    }

    .eh_frame ALIGN(0x1000) : AT(ADDR(.eh_frame) - KERNEL_VMA) {
        __eh_frame_start = .;
        *(.eh_frame*)
        __eh_frame_end = .;
    }

    .data ALIGN(0x1000) : AT(ADDR(.data) - KERNEL_VMA) { *(.data*) }

    .bss ALIGN(0x1000) : AT(ADDR(.bss) - KERNEL_VMA) {
        __bss_start = .;
        *(COMMON)
        *(.bss*)
        __bss_end = .;
    }

    __kernel_top = .;

    /DISCARD/ : { *(.note.gnu.*) }
}