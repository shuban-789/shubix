ENTRY(_start)

SECTIONS
{
    . = 0x8000;
    __start = .;
    __text_start = .;
    .text :
    {
        KEEP(*(.text.boot))
        KEEP(*(.text.vectors))
        *(.text*)
    }
    . = ALIGN(4096);
    __text_end = .;
    __rodata_start = .;
    .rodata : { *(.rodata*) }
    .ARM.extab : { *(.ARM.extab*) }
    .ARM.exidx :
    {
        __exidx_start = .;
        *(.ARM.exidx*)
        __exidx_end = .;
    }
    . = ALIGN(4096);
    __rodata_end = .;
    __data_start = .;
    .data : { *(.data*) *(.got*) }
    . = ALIGN(4096);
    __data_end = .;
    __bss_start = .;
    .bss (NOLOAD) :
    {
        *(.bss*)
        *(COMMON)
        . = ALIGN(4096);
    }
    __bss_end = .;
    __end = .;
    /DISCARD/ : { *(.comment) *(.note*) }
}
