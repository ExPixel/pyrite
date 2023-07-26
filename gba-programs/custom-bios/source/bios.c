#include "common.h"

void reset_handler() {
    asm volatile (
        "ldr lr, =swi_SoftReset\n\t"
        "bx lr                  \n\t"

        : /* NO OUTPUTS */
        : /* NO INPUTS*/
        : /* NO CLOBBERS (we just overwrite) */
    );
}

void irq_handler() {
}

void swi_Debug(int arg0, int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    *((volatile int*)0x02000000) = arg0;
    SWI_RETURN();
}

// SWI 00h (GBA/NDS7/NDS9) - SoftReset
// Clears 200h bytes of RAM (containing stacks, and BIOS IRQ vector/flags), initializes system, supervisor, and irq stack pointers, sets R0-R12, LR_svc, SPSR_svc, LR_irq, and SPSR_irq to zero, and enters system mode.
// Note that the NDS9 stack registers are hardcoded (the DTCM base should be set to the default setting of 0800000h). The NDS9 function additionally flushes caches and write buffer, and sets the CP15 control register to 12078h.
//   Host  sp_svc    sp_irq    sp_sys    zerofilled area       return address
//   GBA   3007FE0h  3007FA0h  3007F00h  [3007E00h..3007FFFh]  Flag[3007FFAh]
//   NDS7  380FFDCh  380FFB0h  380FF00h  [380FE00h..380FFFFh]  Addr[27FFE34h]
//   NDS9  0803FC0h  0803FA0h  0803EC0h  [DTCM+3E00h..3FFFh]   Addr[27FFE24h]
// The NDS7/NDS9 return addresses at [27FFE34h/27FFE24h] are usually containing copies of Cartridge Header [034h/024h] entry points, which may select ARM/THUMB state via bit0. The GBA return address 8bit flag is interpreted as 00h=8000000h (ROM), or 01h-FFh=2000000h (RAM), entered in ARM state.
// Note: The reset is applied only to the CPU that has executed the SWI (ie. on the NDS, the other CPU will remain unaffected).
// Return: Does not return to calling procedure, instead, loads the above return address into R14, and then jumps to that address by a "BX R14" opcode.
__asm__ (
    ".global swi_SoftReset \n\t"
    "swi_SoftReset:        \n\t"

    ".equ MODE_MASK,    0x1F\n\t"
    ".equ MODE_SYS,     0x1F\n\t"
    ".equ MODE_IRQ,     0x12\n\t"
    ".equ MODE_SVC,     0x13\n\t"
    ".equ MODE_SYS,     0x1F\n\t"

    "mrs r1, cpsr           \n\t"   // copy CPSR into r1

    "bic r0, r1, #MODE_MASK \n\t" 
    "orr r0, r0, #MODE_IRQ  \n\t"
    "msr cpsr, r0           \n\t"
    "ldr sp, =0x3007FA0     \n\t"   // sp_irq = 0x3007FA0
    "mov lr, #0             \n\t"   // lr_irq = 0
    "msr spsr, lr           \n\t"   // spsr_irq = 0

    "bic r0, r1, #MODE_MASK \n\t"
    "orr r0, r0, #MODE_SVC  \n\t"
    "msr cpsr, r0           \n\t"
    "ldr sp, =0x3007FE0     \n\t"   // sp_svc = 0x3007FE0
    "mov lr, #0             \n\t"   // lr_svc = 0
    "msr spsr, lr           \n\t"   // spsr_svc = 0

    "ldr r0, =#0x3007E00    \n\t"
    "ldr r1, =#0x0          \n\t"
    "ldr r2, =#0x200        \n\t"
    "ldr r4, =ep_memset     \n\t"
    "mov lr, pc             \n\t"
    "bx r4                  \n\t"

    "bic r0, r1, #MODE_MASK \n\t"
    "orr r0, r0, #MODE_SYS  \n\t"
    "msr cpsr, r0           \n\t"
    "ldr sp, =0x3007F00     \n\t"   // sp_sys = 0x3007F00

    "mov  r0, #0            \n\t"
    "mov  r1, #0            \n\t"
    "mov  r2, #0            \n\t"
    "mov  r3, #0            \n\t"
    "mov  r4, #0            \n\t"
    "mov  r5, #0            \n\t"
    "mov  r6, #0            \n\t"
    "mov  r7, #0            \n\t"
    "mov  r8, #0            \n\t"
    "mov  r9, #0            \n\t"
    "mov r10, #0            \n\t"
    "mov r11, #0            \n\t"
    "mov r12, #0            \n\t"
    "ldr lr, =0x08000000    \n\t"
    "bx lr                  \n\t"

    "mov r0, #0             \n\t"   //  Some padding used for testing.
    "bx r0                  \n\t"
);

void swi_RegisterRamReset(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Halt(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Stop_or_Sleep(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_IntrWait(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_VBlankIntrWait(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Div(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_DivArm(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Sqrt(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_ArcTan(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_ArcTan2(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_CpuSet(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_CpuFastSet(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_GetBiosChecksum(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_BgAffineSet(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_ObjAffineSet(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_BitUnPack(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_LZ77UnCompReadNormalWrite8bit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_LZ77UnCompReadNormalWrite16bit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_HuffUnCompReadNormal(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_RLUnCompReadNormalWrite8bit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_RLUnCompReadNormalWrite16bit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Diff8bitUnFilterWrite8bit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Diff8bitUnFilterWrite16bit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_Diff16bitUnFilter(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundBias(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundDriverInit(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundDriverMode(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundDriverMain(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundDriverVSync(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundChannelClear(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_MidiKey2Freq(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundWhatever0(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundWhatever1(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundWhatever2(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundWhatever3(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundWhatever4(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_MultiBoot(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_HardReset(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_CustomHalt(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundDriverVSyncOff(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundDriverVSyncOn(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}

void swi_SoundGetJumpList(int UNUSED(arg0), int UNUSED(arg1), int UNUSED(arg2), int UNUSED(arg3)) {
    SWI_RETURN();
}