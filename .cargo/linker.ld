OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("x86-64")
ENTRY(kernel_main)

SECTIONS
{
    PROVIDE(__executable_start = SEGMENT_START("text-segment", 0xFFFFFFFF80200000));
    . = SEGMENT_START("text-segment", 0xFFFFFFFF80200000) + SIZEOF_HEADERS;
    . = ALIGN(4096);

    .text ALIGN(4096) :
    {
        *(.text* .text.*)
    }

    . = ALIGN(4096);

    .rodata ALIGN(4096) :
    {
        *(.rodata* .rodata.*)
    }

    . = ALIGN(4096);

    .data ALIGN(4096) :
    {
        *(.data* .data.*)
    }

    . = ALIGN(4096);
    __bss_start = .;
    .bss ALIGN(4096) :
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