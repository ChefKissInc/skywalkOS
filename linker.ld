OUTPUT_FORMAT(elf64-x86-64)

KERNEL_VMA = 0xFFFFFFFF80000000;
KERNEL_LMA = 0x200000;

SECTIONS {
    PROVIDE(__executable_start = KERNEL_VMA + KERNEL_LMA);
    . = KERNEL_VMA + KERNEL_LMA + SIZEOF_HEADERS;

    .kaboom ALIGN(0x1000) : AT(ADDR(.kaboom) - KERNEL_VMA) { *(.kaboom) }

    .text ALIGN(0x1000) : AT(ADDR(.text) - KERNEL_VMA) { *(.text*) }

    .rodata ALIGN(0x1000) : AT(ADDR(.rodata) - KERNEL_VMA) { *(.rodata*) }

    .init_array ALIGN(0x1000) : AT(ADDR(.init_array) - KERNEL_VMA) {
        __initArrayStart = .;
        *(.init_array*)
        __initArrayEnd = .;
    }

    .data ALIGN(0x1000) : AT(ADDR(.data) - KERNEL_VMA) { *(.data*) }

    .bss ALIGN(0x1000) : AT(ADDR(.bss) - KERNEL_VMA) {
        __bssStart = .;
        *(COMMON)
        *(.bss*)
        __bssEnd = .;
    }

    kernelTop = .;
}