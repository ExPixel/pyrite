.section ".init"

.global _start
.align 4
.arm
_start:
    b ev_reset
    b ev_undefined_instruction
    b ev_software_interrupt
    b ev_prefetch_abort
    b ev_data_abort
    b ev_adress_exceeeds_26bit
    b ev_irq_interrupt
    b ev_fiq_interrupt

.section ".text"
ev_reset:
    ldr     r0, =swi_soft_reset
    bx      r0
ev_undefined_instruction:
    movs    pc, lr
ev_software_interrupt:
    movs    pc, lr
ev_prefetch_abort:
    subs    pc, lr, #4
ev_irq_interrupt:
    subs    pc, lr, #4

ev_fiq_interrupt:
    subs    pc, lr, #4

ev_data_abort:
    subs    pc, lr, #8          @ this would retry the instruction, GBA has no data aborts though
ev_adress_exceeeds_26bit:
    subs    pc, lr, #4
