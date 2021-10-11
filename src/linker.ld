ENTRY(kernel_main)
OUTPUT_ARCH(x86-64)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_VIRT_OFF = 0xFFFFFFFF80000000;

SECTIONS
{
    PROVIDE(__executable_start = SEGMENT_START("text-segment", KERNEL_VIRT_OFF + 0x200000));
    . = SEGMENT_START("text-segment", KERNEL_VIRT_OFF + 0x200000) + SIZEOF_HEADERS;
    . = ALIGN(4K);

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VIRT_OFF)
    {
        *(.text*)
    }

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

    __kernel_top = .;

    /DISCARD/ :
    {
        *(.comment*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }
}