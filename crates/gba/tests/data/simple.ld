ENTRY(_start);
MEMORY
{
    ROM (rx)    : ORIGIN = 0x08000000, LENGTH = 16K
}

SECTIONS
{
    . = ORIGIN(ROM);

    /* Place special section .text.prologue before everything else */
    .text : {
        . = ALIGN(4);
        *(.text.prologue);
        *(.text*);
        . = ALIGN(4);
    } >ROM

    .rodata : {
        . = ALIGN(4);
        *(.rodata*);
    } >ROM

    /* Remove the note that may be placed before the code by LD */
    /DISCARD/ : {
        *(.note.gnu.build-id);
        *(.ARM.attributes);
    }
}
