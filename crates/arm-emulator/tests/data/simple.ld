ENTRY(_start);
SECTIONS
{
    . = 0x0;

    /* Place special section .text.prologue before everything else */
    .text : {
        . = ALIGN(4);
        *(.text.prologue);
        *(.text*);
        . = ALIGN(4);
    }

    /* Output the data sections */
    .data : {
        . = ALIGN(4);
        *(.data*);
    }

    .rodata : {
        . = ALIGN(4);
        *(.rodata*);
    }

    /* The BSS section for uninitialized data */
    .bss : {
        . = ALIGN(4);
        __bss_start = .;
        *(COMMON);
        *(.bss);
        . = ALIGN(4);
        __bss_end = .;
    }

    /* Size of the BSS section in case it is needed */
    __bss_size = ((__bss_end)-(__bss_start));

    /* Remove the note that may be placed before the code by LD */
    /DISCARD/ : {
        *(.note.gnu.build-id);
        *(.ARM.attributes);
    }
}
