use arm::emu::{AccessType, Cpu, Memory, RotateRightExtended, Waitstates};
use byteorder::{ByteOrder, LittleEndian};
use util::bits::BitOps;

use crate::hardware::GbaMemoryMappedHardware;

impl GbaMemoryMappedHardware {
    pub fn view8(&self, _address: u32) -> u8 {
        0
    }

    pub fn view16(&self, _address: u32) -> u16 {
        0
    }

    pub fn view32(&self, address: u32) -> u32 {
        let address = address & !0x3;
        match address >> 24 {
            REGION_BIOS if address < 0x4000 => {
                LittleEndian::read_u32(&self.bios[address as usize..])
            }
            // FIXME implement enable/disable from SystemControl
            REGION_EWRAM => LittleEndian::read_u32(&self.ewram[(address & EWRAM_MASK) as usize..]),
            // FIXME implement enable/disable from SystemControl
            REGION_IWRAM => LittleEndian::read_u32(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => 0,
            REGION_PAL => self.palram.load32(address),
            REGION_VRAM => LittleEndian::read_u32(&self.vram[vram_offset(address)..]),
            REGION_OAM => LittleEndian::read_u32(&self.oam[(address & OAM_MASK) as usize..]),

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                LittleEndian::read_u32(&self.gamepak[(address as usize & self.gamepak_mask)..])
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                LittleEndian::read_u32(&self.gamepak[(address as usize & self.gamepak_mask)..])
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                LittleEndian::read_u32(&self.gamepak[(address as usize & self.gamepak_mask)..])
            }
            REGION_SRAM => 0,
            _ => 0,
        }
    }

    fn ioreg_load32(&mut self, address: u32) -> u32 {
        0
    }

    fn ioreg_load16(&mut self, address: u32) -> u16 {
        0
    }

    fn ioreg_load8(&mut self, address: u32) -> u8 {
        0
    }

    fn ioreg_store32(&mut self, address: u32, value: u32) {}
    fn ioreg_store16(&mut self, address: u32, value: u32) {}
    fn ioreg_store8(&mut self, address: u32, value: u32) {}

    fn gamepak_load32<const AREA: usize>(
        &mut self,
        address: u32,
        access_type: AccessType,
        wait: &mut Waitstates,
    ) -> u32 {
        *wait += if access_type == AccessType::Sequential {
            self.system_control.waitstates.gamepak[AREA].1
        } else {
            self.system_control.waitstates.gamepak[AREA].0
        };
        *wait += self.system_control.waitstates.gamepak[AREA].1;
        LittleEndian::read_u32(&self.gamepak[(address as usize & self.gamepak_mask)..])
    }

    fn gamepak_load16<const AREA: usize>(
        &mut self,
        address: u32,
        access_type: AccessType,
        wait: &mut Waitstates,
    ) -> u16 {
        *wait += if access_type == AccessType::Sequential {
            self.system_control.waitstates.gamepak[AREA].1
        } else {
            self.system_control.waitstates.gamepak[AREA].0
        };
        LittleEndian::read_u16(&self.gamepak[(address as usize & self.gamepak_mask)..])
    }

    fn gamepak_load8<const AREA: usize>(
        &mut self,
        address: u32,
        access_type: AccessType,
        wait: &mut Waitstates,
    ) -> u8 {
        let value = self.gamepak_load16::<AREA>(address & !0x1, access_type, wait);
        if address.get_bit(0) {
            (value >> 8) as u8
        } else {
            value as u8
        }
    }

    fn gamepak_store32<const AREA: usize>(
        &mut self,
        address: u32,
        value: u32,
        access_type: AccessType,
        wait: &mut Waitstates,
    ) {
    }

    fn gamepak_store16<const AREA: usize>(
        &mut self,
        address: u32,
        value: u16,
        access_type: AccessType,
        wait: &mut Waitstates,
    ) {
    }

    fn gamepak_store8<const AREA: usize>(
        &mut self,
        address: u32,
        value: u8,
        access_type: AccessType,
        wait: &mut Waitstates,
    ) {
    }

    fn load_sram8<T>(&mut self, address: u32, wait: &mut Waitstates) -> T
    where
        T: From<u8>,
    {
        *wait += self.system_control.waitstates.sram;
        (0u8).into()
    }

    fn store_sram8(&mut self, address: u32, value: u8, wait: &mut Waitstates) {
        *wait += self.system_control.waitstates.sram;
    }
}

impl Memory for GbaMemoryMappedHardware {
    fn load32(&mut self, address: u32, cpu: &mut Cpu) -> (u32, arm::emu::Waitstates) {
        let address = address & !0x3;
        let mut wait = Waitstates::zero();
        let value = match address >> 24 {
            REGION_BIOS if cpu.registers.read(15) < 0x4008 && address < 0x4000 => {
                LittleEndian::read_u32(&self.bios[address as usize..])
            }
            // FIXME implement enable/disable from SystemControl
            REGION_EWRAM => {
                wait += self.system_control.waitstates.ewram + self.system_control.waitstates.ewram;
                LittleEndian::read_u32(&self.ewram[(address & EWRAM_MASK) as usize..])
            }
            // FIXME implement enable/disable from SystemControl
            REGION_IWRAM => LittleEndian::read_u32(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => self.ioreg_load32(address),
            REGION_PAL => {
                wait = Waitstates::one();
                self.palram.load32(address)
            }
            REGION_VRAM => {
                wait = Waitstates::one();
                LittleEndian::read_u32(&self.vram[vram_offset(address)..])
            }
            REGION_OAM => LittleEndian::read_u32(&self.oam[(address & OAM_MASK) as usize..]),

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                self.gamepak_load32::<0>(address, cpu.access_type(), &mut wait)
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                self.gamepak_load32::<1>(address, cpu.access_type(), &mut wait)
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                self.gamepak_load32::<2>(address, cpu.access_type(), &mut wait)
            }
            REGION_SRAM => self
                .load_sram8::<u32>(address, &mut wait)
                // Repeats the byte across the word (e.g. 0xEF -> 0xEFEFEFEF)
                .wrapping_mul(0x01010101u32),

            _ => {
                tracing::debug!("32-bit read from unused memory: [0x{address:08X}]");
                self.last_read_value
            }
        };
        self.last_read_value = value;
        (value, wait)
    }

    fn load16(&mut self, address: u32, cpu: &mut Cpu) -> (u16, arm::emu::Waitstates) {
        let address = address & !0x1;
        let mut wait = Waitstates::zero();
        let value = match address >> 24 {
            REGION_BIOS if cpu.next_execution_address() < 0x4000 && address < 0x4000 => {
                LittleEndian::read_u16(&self.bios[address as usize..])
            }
            // FIXME implement enable/disable from SystemControl
            REGION_EWRAM => {
                wait += self.system_control.waitstates.ewram + self.system_control.waitstates.ewram;
                LittleEndian::read_u16(&self.ewram[(address & EWRAM_MASK) as usize..])
            }
            // FIXME implement enable/disable from SystemControl
            REGION_IWRAM => LittleEndian::read_u16(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => self.ioreg_load16(address),
            REGION_PAL => self.palram.load16(address),
            REGION_VRAM => LittleEndian::read_u16(&self.vram[vram_offset(address)..]),
            REGION_OAM => LittleEndian::read_u16(&self.oam[(address & OAM_MASK) as usize..]),
            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                self.gamepak_load16::<0>(address, cpu.access_type(), &mut wait)
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                self.gamepak_load16::<1>(address, cpu.access_type(), &mut wait)
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                self.gamepak_load16::<2>(address, cpu.access_type(), &mut wait)
            }
            REGION_SRAM => self
                .load_sram8::<u16>(address, &mut wait)
                // Repeats the byte across the halfword (e.g. 0xEF -> 0xEFEF)
                .wrapping_mul(0x0101u16),
            _ => {
                tracing::debug!("16-bit read from unused memory: [0x{address:08X}]");
                self.last_read_value as u16
            }
        };
        self.last_read_value = value as u32;
        (value, wait)
    }

    fn load8(&mut self, address: u32, cpu: &mut Cpu) -> (u8, arm::emu::Waitstates) {
        let mut wait = Waitstates::zero();
        let value = match address >> 24 {
            0x0 if cpu.next_execution_address() < 0x4000 && address < 0x4000 => {
                self.bios[address as usize]
            }
            // FIXME implement enable/disable from SystemControl
            REGION_EWRAM => {
                wait += self.system_control.waitstates.ewram + self.system_control.waitstates.ewram;
                self.ewram[(address & EWRAM_MASK) as usize]
            }
            // FIXME implement enable/disable from SystemControl
            REGION_IWRAM => self.iwram[(address & IWRAM_MASK) as usize],
            REGION_IOREGS => self.ioreg_load8(address),
            REGION_PAL => self.palram.load8(address),
            REGION_VRAM => self.vram[vram_offset(address)],
            REGION_OAM => self.oam[(address & OAM_MASK) as usize],
            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                self.gamepak_load8::<0>(address, cpu.access_type(), &mut wait)
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                self.gamepak_load8::<1>(address, cpu.access_type(), &mut wait)
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                self.gamepak_load8::<2>(address, cpu.access_type(), &mut wait)
            }
            REGION_SRAM => self.load_sram8::<u8>(address, &mut wait),
            _ => {
                tracing::debug!("8-bit read from unused memory: [0x{address:08X}]");
                self.last_read_value as u8
            }
        };
        self.last_read_value = value as u32;
        (value, wait)
    }

    fn store32(&mut self, address: u32, value: u32, cpu: &mut Cpu) -> arm::emu::Waitstates {
        let address = address & !0x3;
        let mut wait = Waitstates::zero();
        match address >> 24 {
            // FIXME implement enable/disable from SystemControl
            REGION_EWRAM => {
                wait += self.system_control.waitstates.ewram + self.system_control.waitstates.ewram;
                LittleEndian::write_u32(&mut self.ewram[(address & EWRAM_MASK) as usize..], value);
            }
            // FIXME implement enable/disable from SystemControl
            REGION_IWRAM => {
                LittleEndian::write_u32(&mut self.iwram[(address & IWRAM_MASK) as usize..], value);
            }
            REGION_IOREGS => self.ioreg_store32(address, value),
            REGION_PAL => {
                wait = Waitstates::one();
                self.palram.store32(address, value);
            }
            REGION_VRAM => {
                wait = Waitstates::one();
                LittleEndian::write_u32(&mut self.vram[vram_offset(address)..], value);
            }
            REGION_OAM => {
                LittleEndian::write_u32(&mut self.oam[(address & OAM_MASK) as usize..], value)
            }
            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                self.gamepak_store32::<0>(address, value, cpu.access_type(), &mut wait);
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                self.gamepak_store32::<1>(address, value, cpu.access_type(), &mut wait);
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                self.gamepak_store32::<2>(address, value, cpu.access_type(), &mut wait);
            }
            REGION_SRAM => self.store_sram8(
                address,
                value.rotate_right((address & 0x7) * 8) as u8,
                &mut wait,
            ),
            _ => {
                tracing::debug!("32-bit write to unused memory: [0x{address:08X}] = 0x{value:08X}");
            }
        }
        wait
    }

    fn store16(&mut self, address: u32, value: u16, cpu: &mut Cpu) -> arm::emu::Waitstates {
        let address = address & !0x1;
        let wait = Waitstates::zero();
        match address >> 24 {
            _ => {
                tracing::debug!("16-bit write to unused memory: [0x{address:08X}] = 0x{value:04X}");
            }
        }
        wait
    }

    fn store8(&mut self, address: u32, value: u8, cpu: &mut Cpu) -> arm::emu::Waitstates {
        let wait = Waitstates::zero();
        match address >> 24 {
            _ => {
                tracing::debug!("8-bit write to unused memory: [0x{address:08X}] = 0x{value:02X}");
            }
        }
        wait
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Converts an address in the range [0x06000000, 0x06FFFFFF] into an offset in VRAM accounting
/// for VRAM mirroring.
const fn vram_offset(address: u32) -> usize {
    // Even though VRAM is sized 96K (64K+32K), it is repeated in steps of 128K (64K+32K+32K,
    // the two 32K blocks itself being mirrors of each other).
    let vram128 = address % (128 * 1024); // offset in a 128KB block

    if vram128 >= (96 * 1024) {
        // this means that this address is in the later 32KB block so we just subtract 32KB to
        // mirror the previous one:
        vram128 as usize - (32 * 1024)
    } else {
        vram128 as usize
    }
}

pub const REGION_BIOS: u32 = 0x0;
pub const REGION_UNUSED_1: u32 = 0x1;
pub const REGION_EWRAM: u32 = 0x2;
pub const REGION_IWRAM: u32 = 0x3;
pub const REGION_IOREGS: u32 = 0x4;
pub const REGION_PAL: u32 = 0x5;
pub const REGION_VRAM: u32 = 0x6;
pub const REGION_OAM: u32 = 0x7;
pub const REGION_GAMEPAK0_LO: u32 = 0x8;
pub const REGION_GAMEPAK0_HI: u32 = 0x9;
pub const REGION_GAMEPAK1_LO: u32 = 0xA;
pub const REGION_GAMEPAK1_HI: u32 = 0xB;
pub const REGION_GAMEPAK2_LO: u32 = 0xC;
pub const REGION_GAMEPAK2_HI: u32 = 0xD;
pub const REGION_SRAM: u32 = 0xE;

pub const BIOS_SIZE: usize = 0x4000;
pub const EWRAM_SIZE: usize = 0x40000;
pub const IWRAM_SIZE: usize = 0x8000;
pub const PAL_SIZE: usize = 0x400;
pub const VRAM_SIZE: usize = 0x18000;
pub const OAM_SIZE: usize = 0x400;
pub const IOREGS_SIZE: usize = 0x20A;

pub const EWRAM_MASK: u32 = 0x3FFFF;
pub const IWRAM_MASK: u32 = 0x7FFF;
pub const PAL_MASK: u32 = 0x3FF;
pub const OAM_MASK: u32 = 0x3FF;
pub const ROM_MAX_MASK: u32 = 0xFFFFFF;

pub trait IoRegister<T: BitOps>: Copy + From<T> {
    fn read(self) -> T;
    fn write(&mut self, value: T);

    #[inline(always)]
    fn write8_using_address(&mut self, address: u32, value: u8)
    where
        T: From<u8>,
    {
        let mask = (T::BITS / 8) - 1;
        let offset = (address & mask) * 8;
        let original = self.read();
        self.write(original.put_bit_range(offset..(offset + 16), value.into()));
    }

    #[inline(always)]
    fn write16_using_address(&mut self, address: u32, value: u16)
    where
        T: From<u16>,
    {
        let mask = (T::BITS / 16) - 1;
        let offset = (address & mask) * 16;
        let original = self.read();
        self.write(original.put_bit_range(offset..(offset + 16), value.into()));
    }

    #[inline(always)]
    fn write16_lo(&mut self, value: u16)
    where
        T: From<u16>,
    {
        let original = self.read();
        self.write(original.put_bit_range(0..16, value.into()));
    }

    #[inline(always)]
    fn write16_hi(&mut self, value: u16)
    where
        T: From<u16>,
    {
        let original = self.read();
        self.write(original.put_bit_range(16..32, value.into()));
    }
}
