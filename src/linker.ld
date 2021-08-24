ENTRY(kernel_main)

KERNEL_VIRT_OFF = 0xFFFFFFFF80000000;

SECTIONS {
    . = KERNEL_VIRT_OFF + 0x200000;

    .kaboom ALIGN(4K) : AT(ADDR(.kaboom) - KERNEL_VIRT_OFF) { KEEP(*(.kaboom)) }

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VIRT_OFF) { KEEP(*(.text*)) }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_VIRT_OFF) { KEEP(*(.rodata*)) }

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_VIRT_OFF) { KEEP(*(.data*)) }

    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_VIRT_OFF) {
        __bss_start = .;
        KEEP(*(COMMON))
        KEEP(*(.bss*))
        __bss_end = .;
    }

    __kernel_top = .;

    /DISCARD/ : { *(.note.gnu.*) }
}