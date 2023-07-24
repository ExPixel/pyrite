use arm::emu::Waitstates;
use pyrite_derive::IoRegister;

#[derive(Default)]
pub struct SystemControl {
    pub waitcnt: RegWaitcnt,
    pub internal_memory_control: RegInternalMemoryControl,
    pub waitstates: SystemWaitstates,
}

impl SystemControl {
    pub fn write_waitcnt(&mut self, waitcnt: RegWaitcnt) {
        self.waitcnt = waitcnt;
        self.update_waitstates();
    }

    pub fn write_internal_memory_control(
        &mut self,
        internal_memory_control: RegInternalMemoryControl,
    ) {
        self.internal_memory_control = internal_memory_control;
        self.update_waitstates();
    }

    pub fn update_waitstates(&mut self) {
        self.waitstates.sram = match self.waitcnt.sram_wait_control() {
            0 => Waitstates::from(4u32),
            1 => Waitstates::from(3u32),
            2 => Waitstates::from(2u32),
            3 => Waitstates::from(8u32),
            _ => unreachable!(),
        };

        self.waitstates.gamepak[0].0 = match self.waitcnt.waitstate_0_first_access() {
            0 => Waitstates::from(4u32),
            1 => Waitstates::from(3u32),
            2 => Waitstates::from(2u32),
            3 => Waitstates::from(8u32),
            _ => unreachable!(),
        };

        self.waitstates.gamepak[0].1 = match self.waitcnt.waitstate_0_second_access() {
            0 => Waitstates::from(2u32),
            1 => Waitstates::from(1u32),
            _ => unreachable!(),
        };

        self.waitstates.gamepak[1].0 = match self.waitcnt.waitstate_1_first_access() {
            0 => Waitstates::from(4u32),
            1 => Waitstates::from(3u32),
            2 => Waitstates::from(2u32),
            3 => Waitstates::from(8u32),
            _ => unreachable!(),
        };

        self.waitstates.gamepak[1].1 = match self.waitcnt.waitstate_1_second_access() {
            0 => Waitstates::from(4u32),
            1 => Waitstates::from(1u32),
            _ => unreachable!(),
        };

        self.waitstates.gamepak[2].0 = match self.waitcnt.waitstate_2_first_access() {
            0 => Waitstates::from(4u32),
            1 => Waitstates::from(3u32),
            2 => Waitstates::from(2u32),
            3 => Waitstates::from(8u32),
            _ => unreachable!(),
        };

        self.waitstates.gamepak[2].1 = match self.waitcnt.waitstate_2_second_access() {
            0 => Waitstates::from(8u32),
            1 => Waitstates::from(1u32),
            _ => unreachable!(),
        };

        self.waitstates.ewram = match self.internal_memory_control.wait_control_ewram() {
            value @ 0..=14 => Waitstates::from(15u32 - value),
            15 => {
                tracing::warn!("EWRAM waitstates set to 0, this would lock up real hardware");
                Waitstates::from(0)
            }
            _ => unreachable!(),
        };

        tracing::debug!(waitstates = debug(&self.waitstates), "waitstates updated");
    }
}

#[derive(Debug, Default)]
pub struct SystemWaitstates {
    pub sram: Waitstates,
    pub gamepak: [(
        /*  first access */ Waitstates,
        /* second access */ Waitstates,
    ); 3],
    pub ewram: Waitstates,
}

/// 4000204h - WAITCNT - Waitstate Control (R/W)
/// This register is used to configure game pak access timings. The game pak ROM
/// is mirrored to three address regions at 08000000h, 0A000000h, and 0C000000h,
/// these areas are called Wait State 0-2. Different access timings may be assigned
/// to each area (this might be useful in case that a game pak contains several ROM
/// chips with different access times each).
///
/// ```ignore
///   Bit   Expl.
///   0-1   SRAM Wait Control          (0..3 = 4,3,2,8 cycles)
///   2-3   Wait State 0 First Access  (0..3 = 4,3,2,8 cycles)
///   4     Wait State 0 Second Access (0..1 = 2,1 cycles)
///   5-6   Wait State 1 First Access  (0..3 = 4,3,2,8 cycles)
///   7     Wait State 1 Second Access (0..1 = 4,1 cycles; unlike above WS0)
///   8-9   Wait State 2 First Access  (0..3 = 4,3,2,8 cycles)
///   10    Wait State 2 Second Access (0..1 = 8,1 cycles; unlike above WS0,WS1)
///   11-12 PHI Terminal Output        (0..3 = Disable, 4.19MHz, 8.38MHz, 16.78MHz)
///   13    Not used
///   14    Game Pak Prefetch Buffer (Pipe) (0=Disable, 1=Enable)
///   15    Game Pak Type Flag  (Read Only) (0=GBA, 1=CGB) (IN35 signal)
///   16-31 Not used
/// ```
///
/// At startup, the default setting is 0000h. Currently manufactured cartridges are
/// using the following settings: WS0/ROM=3,1 clks; SRAM=8 clks; WS2/EEPROM: 8,8 clks;
/// prefetch enabled; that is, WAITCNT=4317h, for more info see "GBA Cartridges" chapter.
#[derive(IoRegister, Copy, Clone)]
#[field(sram_wait_control: u32 = 0..=1)]
#[field(waitstate_0_first_access: u32 = 2..=3)]
#[field(waitstate_0_second_access: u32 = 4)]
#[field(waitstate_1_first_access: u32 = 5..=6)]
#[field(waitstate_1_second_access: u32 = 7)]
#[field(waitstate_2_first_access: u32 = 8..=9)]
#[field(waitstate_2_second_access: u32 = 10)]
#[field(phi_terminal_output: u32 = 11..=12)]
#[field(gamepak_prefetch_buffer_enabled: bool = 14)]
#[field(gamepak_type_flag: u32 = 15)]
pub struct RegWaitcnt {
    value: u32,
}

/// 4000800h - 32bit - Undocumented - Internal Memory Control (R/W)
/// Supported by GBA and GBA SP only - NOT supported by DS (even in GBA mode).
/// Also supported by GBA Micro - but crashes on "overclocked" WRAM setting.
/// Initialized to 0D000020h (by hardware). Unlike all other I/O registers, this
/// register is mirrored across the whole I/O area (in increments of 64K, ie. at 4000800h, 4010800h, 4020800h, ..., 4FF0800h)
///
/// ```ignore
///   Bit   Expl.
///   0     Disable 32K+256K WRAM (0=Normal, 1=Disable) (when off: empty/prefetch)
///          From endrift: bit0 swaps 00000000h-01FFFFFFh and 02000000h-03FFFFFFh
///          in GBA mode (but keeps BIOS protection)
///   1     Unknown          (Read/Write-able)
///   2     Unknown          (Read/Write-able)
///   3     Unknown, CGB?    (Read/Write-able)
///          From shinyquagsire23: bit3 seems to disable the CGB bootrom (carts
///          without SRAM will typically boot with Nintendo logo skipped, and
///          carts with SRAM will typically crash somehow)
///   4     Unused (0)
///   5     Enable 256K WRAM (0=Disable, 1=Normal) (when off: mirror of 32K WRAM)
///   6-23  Unused (0)
///   24-27 Wait Control WRAM 256K (0-14 = 15..1 Waitstates, 15=Lockup)
///   28-31 Unknown          (Read/Write-able)
/// ```
///
/// The default value 0Dh in Bits 24-27 selects 2 waitstates for 256K WRAM (ie. 3/3/6 cycles 8/16/32bit accesses).
/// The fastest possible setting would be 0Eh (1 waitstate, 2/2/4 cycles for 8/16/32bit), that works on GBA and GBA SP only,
/// the GBA Micro locks up with that setting (it's on-chip RAM is too slow, and works only with 2 or more waitstates).
#[derive(IoRegister, Copy, Clone)]
#[field(ram_disabled: bool = 0)]
#[field(external_ram_enabled: bool = 5)]
#[field(wait_control_ewram: u32 = 24..=27)]
pub struct RegInternalMemoryControl {
    value: u32,
}

impl RegInternalMemoryControl {
    pub const DEFAULT: RegInternalMemoryControl = RegInternalMemoryControl::new(0x0D000020);
}
