OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("x86-64")
ENTRY(kernel_main)

KERNEL_VIRT_OFF = 0xFFFFFFFF80000000;

SECTIONS
{
    PROVIDE(__executable_start = SEGMENT_START("text-segment", KERNEL_VIRT_OFF + 0x200000));
    . = SEGMENT_START("text-segment", KERNEL_VIRT_OFF + 0x200000) + SIZEOF_HEADERS;
    . = ALIGN(4096);

    .text ALIGN(4096) : AT(ADDR(.text) - KERNEL_VIRT_OFF)
    {
        *(.text* .text.*)
    }

    . = ALIGN(4096);

    .rodata ALIGN(4096) : AT(ADDR(.rodata) - KERNEL_VIRT_OFF)
    {
        *(.rodata* .rodata.*)
    }

    . = ALIGN(4096);

    .data ALIGN(4096) : AT(ADDR(.data) - KERNEL_VIRT_OFF)
    {
        *(.data* .data.*)
    }

    . = ALIGN(4096);
    __bss_start = .;
    .bss ALIGN(4096) : AT(ADDR(.bss) - KERNEL_VIRT_OFF)
    {
        *(.bss* .bss.*)
        *(COMMON)
    }

    . = ALIGN(4096);
    __bss_end = .;

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