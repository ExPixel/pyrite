.equ MODE_MASK,    0x1F
.equ MODE_SYS,     0x1F

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
    ldr r0, =reset_handler
    bx r0
ev_software_interrupt:
    push {r11, r12, lr}
    mrs r11, spsr
    push {r11}
    
    ldr r12, [lr, #-4]
    and r12, #0xFF
    mrs r11, cpsr
    msr spsr, r11
    bic r11, r11, #MODE_MASK
    orr r11, r11, #MODE_SYS
    msr cpsr, r11

    ldr r11, =swi_handlers
    add r11, r12, LSL #2
    ldr r11, [r11]
    mov lr, pc
    bx r11

    pop {r11}
    msr spsr, r11
    pop {r11, r12, lr}
    movs pc, lr
ev_irq_interrupt:
    ldr lr, =irq_handler
    bx lr

ev_undefined_instruction:
    ldr r0, =ev_reset
    bx r0
    movs pc, lr
ev_prefetch_abort:
    ldr r0, =ev_reset
    bx r0
    subs pc, lr, #4
ev_fiq_interrupt:
    ldr r0, =ev_reset
    bx r0
    subs pc, lr, #4
ev_data_abort:
    ldr r0, =ev_reset
    bx r0
    subs pc, lr, #8          @ this would retry the instruction, GBA has no data aborts though
ev_adress_exceeeds_26bit:
    ldr r0, =ev_reset
    bx r0
    subs pc, lr, #4


swi_handlers:
    .word swi_SoftReset                         @ 0x00
    .word swi_RegisterRamReset                  @ 0x01
    .word swi_Halt                              @ 0x02
    .word swi_Stop_or_Sleep                     @ 0x03
    .word swi_IntrWait                          @ 0x04
    .word swi_VBlankIntrWait                    @ 0x05
    .word swi_Div                               @ 0x06
    .word swi_DivArm                            @ 0x07
    .word swi_Sqrt                              @ 0x08
    .word swi_ArcTan                            @ 0x09
    .word swi_ArcTan2                           @ 0x0A
    .word swi_CpuSet                            @ 0x0B
    .word swi_CpuFastSet                        @ 0x0C
    .word swi_GetBiosChecksum                   @ 0x0D
    .word swi_BgAffineSet                       @ 0x0E
    .word swi_ObjAffineSet                      @ 0x0F
    .word swi_BitUnPack                         @ 0x10
    .word swi_LZ77UnCompReadNormalWrite8bit     @ 0x11
    .word swi_LZ77UnCompReadNormalWrite16bit    @ 0x12
    .word swi_HuffUnCompReadNormal              @ 0x13
    .word swi_RLUnCompReadNormalWrite8bit       @ 0x14
    .word swi_RLUnCompReadNormalWrite16bit      @ 0x15
    .word swi_Diff8bitUnFilterWrite8bit         @ 0x16
    .word swi_Diff8bitUnFilterWrite16bit        @ 0x17
    .word swi_Diff16bitUnFilter                 @ 0x18
    .word swi_SoundBias                         @ 0x19
    .word swi_SoundDriverInit                   @ 0x1A
    .word swi_SoundDriverMode                   @ 0x1B
    .word swi_SoundDriverMain                   @ 0x1C
    .word swi_SoundDriverVSync                  @ 0x1D
    .word swi_SoundChannelClear                 @ 0x1E
    .word swi_MidiKey2Freq                      @ 0x1F
    .word swi_SoundWhatever0                    @ 0x20
    .word swi_SoundWhatever1                    @ 0x21
    .word swi_SoundWhatever2                    @ 0x22
    .word swi_SoundWhatever3                    @ 0x23
    .word swi_SoundWhatever4                    @ 0x24
    .word swi_MultiBoot                         @ 0x25
    .word swi_HardReset                         @ 0x26
    .word swi_CustomHalt                        @ 0x27
    .word swi_SoundDriverVSyncOff               @ 0x28
    .word swi_SoundDriverVSyncOn                @ 0x29
    .word swi_SoundGetJumpList                  @ 0x2A
    .word swi_Debug                             @ 0x2B
