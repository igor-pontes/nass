use super::Mapper;
use std::fmt;
use crate::mapper::Mirroring;

const PRG_BANK_SIZE_256: usize = 0x40000;
const PRG_BANK_SIZE_32: usize = 0x8000;
const PRG_BANK_SIZE_16: usize = 0x4000;
const PRG_BANK_SIZE_8: usize = 0x2000; // PRG RAM bank size
const CHR_BANK_SIZE_8: usize = 0x2000;
const CHR_BANK_SIZE_4: usize = 0x1000;

use ChrBanks::*;
use BankType::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Clone, Copy, Debug)]
enum BankType {
    Fixed,
    Switch(usize),
    Null
}

type PrgBanks = (BankType, BankType);

#[derive(Clone, Copy, Debug)]
enum ChrBanks {
    Ram(usize, Option<usize>),
    Rom(usize, Option<usize>) 
}

// MMC1 with 512K(PRG-ROM) was supported by re-using a line from the CHR banking controls. 
// https://www.nesdev.org/wiki/MMC1
pub struct MMC1 {
    sr: u8,
    is_variant: bool,
    chr_addr: ChrBanks,
    prg_rom_addr: PrgBanks,
    prg_ram_addr: usize,
    prg_area: usize,
    prg_ram: [u8; 0x8000],
    prg_rom: Vec<u8>, 
    chr_ram: [u8; 0x20000], // 128KB (2**10 * 128)
    chr_rom: Vec<u8>, 
    mirroring: Mirroring
}

impl fmt::Display for MMC1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MMC1")
    }
}

impl MMC1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        // (No CHR_ROM) or (CHR_ROM == 8) => Using 8KB variant
        let is_rom = chr_rom.len() != 0;
        let chr_addr = if is_rom { Rom(0, None) } else { Ram(0, None) };
        MMC1 {
            sr: 0x10,
            is_variant: !is_rom || chr_rom.len() == 8,
            chr_addr,
            prg_rom_addr: (Switch(0), Fixed),
            prg_ram_addr: 0,
            prg_area: 0,
            mirroring,
            prg_ram: [0; 0x8000],
            prg_rom,
            chr_ram: [0; 0x20000], 
            chr_rom,
        } 
    }

    fn set_variant(&mut self, value: u8, ignore: bool, register: usize) {
        let bank = ((value & 1) as usize) * 0x1000 ;

        if register == 1 && !ignore {
            match self.chr_addr {
                Ram(_, b) => self.chr_addr = Ram(bank * CHR_BANK_SIZE_4, b),
                Rom(_, b) => self.chr_addr = Rom(bank * CHR_BANK_SIZE_4, b),
            } 
        }
        if register == 2 {
            match self.chr_addr {
                Ram(a, Some(_)) => self.chr_addr = Ram(a, Some(bank * CHR_BANK_SIZE_4)),
                Rom(a, Some(_)) => self.chr_addr = Rom(a, Some(bank * CHR_BANK_SIZE_4)),
                _ => (),
            } 
        }

        let prg_bank = ((value & 0x0C) >> 2) as usize;
        self.prg_ram_addr = prg_bank * PRG_BANK_SIZE_8;

        // The 256 KB PRG bank selection applies to all the PRG area(0x6000 - 0xFFFF), including the supposedly "fixed" bank.
        let prg_bank = ((value & 0x10) >> 4) as usize;
        self.prg_area = prg_bank * PRG_BANK_SIZE_256;
    }

    fn set_reg(&mut self, reg: u16, value: u8) {
        match reg {
            0 => { // Control register
                match value & 0x03 {
                    0 => self.mirroring = Mirroring::OneScreenLower,
                    1 => self.mirroring = Mirroring::OneScreenUpper,
                    2 => self.mirroring = Mirroring::Vertical,
                    3 => self.mirroring = Mirroring::Horizontal,
                    _ => ()
                };
                match (value & 0x0C) >> 2 {
                    0 | 1 => self.prg_rom_addr = (Switch(0), Null),
                    2 => self.prg_rom_addr = (Fixed, Switch(0)),
                    3 => self.prg_rom_addr = (Switch(0), Fixed),
                    _ => ()
                }
                match (value & 0x10) >> 4 {
                    0 => self.chr_addr = {
                        if self.chr_rom.len() != 0 {
                            Rom(0, None)
                        } else {
                            Ram(0, None)
                        }
                    },
                    1 => self.chr_addr = {
                        if self.chr_rom.len() != 0 {
                            Rom(0, Some(0x1000))
                        } else {
                            Ram(0, Some(0x1000))
                        }
                    },
                    _ => ()
                }
            },
            1 => { // CHR bank 0 register
                if !self.is_variant {
                    match self.chr_addr {
                        Rom(_, None) => {
                            let bank = (value & 0x1E) as usize;
                            self.chr_addr = Rom(bank * CHR_BANK_SIZE_8, None);
                        },
                        Ram(_, None) => {
                            let bank = (value & 0x1E) as usize;
                            self.chr_addr = Ram(bank * CHR_BANK_SIZE_8, None);
                        },
                        Rom(_, b) => {
                            let bank  = (value & 0x1F) as usize;
                            self.chr_addr = Rom(bank * CHR_BANK_SIZE_4, b);
                        },
                        Ram(_, b) => {
                            let bank  = (value & 0x1F) as usize;
                            self.chr_addr = Ram(bank * CHR_BANK_SIZE_4, b);
                        },
                    }
                } else { // High Address lines are used for different purposes
                    match self.chr_addr {
                        Ram(_, None) | Rom(_, None) => self.set_variant(value, true, 1),
                        _ => self.set_variant(value, false, 1),
                    }
                }
            },
            2 => { // CHR bank 1 register
                if !self.is_variant {
                    match self.chr_addr {
                        Rom(a, Some(_)) => {
                            let bank  = (value & 0x1F) as usize;
                            self.chr_addr = Rom(a, Some(bank * CHR_BANK_SIZE_4));
                        },
                        Ram(a, Some(_)) => {
                            let bank  = (value & 0x1F) as usize;
                            self.chr_addr = Ram(a, Some(bank * CHR_BANK_SIZE_4));
                        },
                        _ => ()
                    }
                } else {
                    match self.chr_addr {
                        Ram(_, Some(_)) | Rom(_, Some(_)) => self.set_variant(value, false, 2),
                        _ => (),
                    }
                }
            }, 
            3 => { // PRG bank register
                match self.prg_rom_addr {
                    (_, Null) => {
                        let bank = (value & 0x0E) as usize;
                        self.prg_rom_addr = (Switch(bank * PRG_BANK_SIZE_32), Null);
                    },
                    (a, Switch(_)) => {
                        let bank  = (value & 0x0F) as usize;
                        self.prg_rom_addr = (a, Switch(bank * PRG_BANK_SIZE_16));
                    },
                    (Switch(_), b) => {
                        let bank  = (value & 0x0F) as usize;
                        self.prg_rom_addr = (Switch(bank * PRG_BANK_SIZE_16), b);
                    }
                    _ => ()
                }
            },
            _ => { panic!("MMC1: Unknown register."); }
        };
    }

    fn update_sr(&mut self, value: u8, addr: u16) {
        if (value & 0x7) != 0 { self.sr = 0x10; return; }
        if self.sr & 1 == 1 {
            self.sr >>= 1;
            self.sr |= (value & 0x1) << 4;
            let reg = ((addr & 0xF000) >> 13) - 4;
            self.set_reg(reg, self.sr);
            self.sr = 0x10;
        } else {
            self.sr >>= 1;
            self.sr |= (value & 0x1) << 5;
        }
    }
}

impl Mapper for MMC1 {
    fn get_mirroring(&self) -> Mirroring { self.mirroring }

    fn read_prg(&self, addr: u16) -> u8 { 
        // log(&format!("[Read PRG] Addr: {addr:#06x} | PRG_RAM_ADDR: {} | PRG_ROM_ADDR: {:?}", self.prg_ram_addr, self.prg_rom_addr));
        if (0x6000..=0x7FFF).contains(&addr) { return self.prg_ram[(addr -  0x6000) as usize + self.prg_ram_addr + self.prg_area] }
        let mut addr = addr as usize - 0x8000;
        let rom_len = self.prg_rom.len();
        if rom_len == 0x4000 && addr >= 0x4000 { return self.prg_rom[addr % 0x4000] }
        match self.prg_rom_addr {
            (Switch(x), Null)  => addr += x + self.prg_area,
            (Switch(x), _) if addr < 0x4000 => addr += x + self.prg_area,
            (Fixed,     _) if addr < 0x4000 => addr += self.prg_area,
            (_, Switch(x)) => addr = addr - PRG_BANK_SIZE_16 + x + self.prg_area,
            (_, Fixed)     => addr = addr + rom_len - 2*PRG_BANK_SIZE_16 + self.prg_area,
            _  => panic!("MMC1: (Null, Null)")
        }
        self.prg_rom[addr]
    }

    fn write_prg(&mut self, addr: u16, val: u8) { 
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr -  0x6000) as usize + self.prg_ram_addr + self.prg_area] = val,
            0x8000..=0xFFFF => self.update_sr(val, addr),
            _ => ()
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        match self.chr_addr {
            Ram(_, Some(x)) if addr >= 0x1000 => self.chr_ram[addr as usize + x - CHR_BANK_SIZE_4],
            Rom(_, Some(x)) if addr >= 0x1000 => self.chr_rom[addr as usize + x - CHR_BANK_SIZE_4],
            Ram(x, _) => self.chr_ram[addr as usize + x],
            Rom(x, _) => self.chr_rom[addr as usize + x],
        }
    }

    fn write_chr(&mut self, addr: u16, val: u8) { 
        match self.chr_addr {
            Ram(_, Some(x)) => self.chr_ram[addr as usize + x - CHR_BANK_SIZE_4] = val,
            Ram(x, _)       => self.chr_ram[addr as usize + x] = val,
            _ => (),
        }
    }
}
