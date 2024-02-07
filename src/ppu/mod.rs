use std::rc::Rc;
use std::cell::RefCell;
mod addr_register;
mod control_register;
mod ppu_mask;
mod ppu_status;
use crate::frame::Frame;
use crate::mapper::*;
pub use self::addr_register::AddrRegister;
pub use self::control_register::ControlRegister;
pub use self::ppu_mask::PPUMask;
pub use self::ppu_status::PPUStatus;
use crate::mapper::Mirroring;

pub struct PPU {
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>,
    pub palette_table: [u8; 0x20],
    pub vram: [u8; 0x800], // Nametables (2kB)
    pub oam_data: [u8; 0x100],
    pub oam_addr: u8,
    addr: AddrRegister,
    temp: u16,
    pub ctrl: ControlRegister,
    mask: PPUMask,
    status: PPUStatus,
    internal_data_buff: u8,
    fine_x: u8,
    scanline: u16,
    pub cycle: usize,
    odd_frame: bool,
    pub frame: Frame,
    pub nmi_ocurred: bool,
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        PPU {
            palette_table: [0; 0x20],
            mapper,
            vram: [0; 0x800],
            oam_data: [0; 0x100],
            oam_addr: 0,
            addr: AddrRegister::new(),
            temp: 0,
            ctrl: ControlRegister::new(),
            mask: PPUMask::new(),
            status: PPUStatus::new(),
            internal_data_buff: 0,
            fine_x: 0,
            scanline: 0,
            cycle: 0,
            odd_frame: false,
            frame: Frame::new(),
            nmi_ocurred: false,
        }
    }

    pub fn step(&mut self)  {
        match self.scanline {
            261..=u16::MAX => {
                if self.cycle > 0 {
                    if self.cycle == 1 { 
                        self.status.set_vblank(false);
                        self.status.set_overflow(false);
                        self.status.set_sprite_hit(false);
                    }

                    if self.mask.rendering() { 
                        if self.cycle % 8 == 0 && self.cycle <= 256 { self.addr.coarse_x_increment(); }
                        if self.cycle == 256 { self.addr.coarse_y_increment(); }
                        if self.cycle == 257 { self.oam_addr = 0; self.addr.set_horizontal(self.temp); }
                        if self.cycle >= 280 && self.cycle <= 304 { self.addr.set_vertical(self.temp); }

                    }

                    if self.cycle == 340 && self.odd_frame && self.mask.rendering() { 
                        self.scanline = 0; 
                        self.cycle = 0; 
                    }
                }
            },
            0..=239 => {
                if self.cycle > 0 {
                    if self.cycle <= 256 {
                        let mut color = 0;
                        if self.mask.show_background() && (self.cycle > 8 || self.mask.show_background_leftmost()) {
                            let v = self.addr.get();
                            let fine_x = 8 - (self.fine_x + ((self.cycle as u8) % 8));
                            let fine_y = (v & 0x7000) >> 12;

                            // https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
                            let tile_addr = 0x2000 | (v & 0x0FFF);
                            let tile = self.vram[self.mirror_vram_addr(tile_addr) as usize];
                            let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                            let attr_data = self.vram[self.mirror_vram_addr(attr_addr) as usize];

                            let half_pattern_table = self.ctrl.get_background_pattern_addr();
                            let color_addr_0 = half_pattern_table | (tile as u16) << 4 | 0 << 3 | fine_y;
                            let color_addr_1 = half_pattern_table | (tile as u16) << 4 | 1 << 3 | fine_y;
                            let color_bit_0 = ( self.mapper.borrow().read_chr(color_addr_0) >> fine_x) & 0x1;
                            let color_bit_1 = ( self.mapper.borrow().read_chr(color_addr_1) >> fine_x) & 0x1;
                            let color_tile = (color_bit_1 << 1) | color_bit_0;

                            let tile_column = (v & 0x001f) as u8;
                            let tile_row = ((v & 0x03e0) >> 5) as u8;
                            let quadrant = (tile_row & 0x2) + ((tile_column & 0x2) >> 1);
                            let offset = quadrant * 2;
                            let attr_color = (attr_data >> offset) & 0x3;
                            color = self.palette_table[(attr_color << 2 | color_tile) as usize];
                        }
                        self.frame.set_pixel(color);
                    }

                    if self.mask.rendering() {
                        if self.cycle % 8 == 0 && self.cycle <= 256 { self.addr.coarse_x_increment(); }
                        if self.cycle == 256 { self.addr.coarse_y_increment(); }
                        if self.cycle == 257 { self.oam_addr = 0; self.addr.set_horizontal(self.temp); }
                    }
                }
            },
            240..=260 => {
                if self.scanline == 241 && self.cycle == 1 { 
                    self.odd_frame = !self.odd_frame;
                    self.status.set_vblank(true);
                    if self.ctrl.generate_nmi() { self.nmi_ocurred = true; }
                }
            }
        }
        self.cycle += 1;
        if self.cycle == 341 { 
            self.scanline = (self.scanline + 1) % 262; 
            self.cycle = 0; 
        }
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        if self.addr.latch() {
            self.fine_x = value & 0x07;
            let value = value >> 3;
            self.temp = (self.temp & 0xFFE0) | (value as u16);
        } else {
            let fine_y_scroll = value & 0x07;
            let coarse_y_scroll = value & 0xF8;
            self.temp = self.temp & 0x8C1F | ( fine_y_scroll as u16 ) << 12 | (coarse_y_scroll as u16) << 5;
        }
        self.addr.toggle_latch();
    }

    pub fn read_status(&mut self) -> u8 {
        let status = self.status.bits();
        self.status.set_vblank(false);
        self.addr.reset_latch();
        status
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value, &mut self.temp);
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_nmi();
        self.ctrl.update(value, &mut self.temp);
        if !before_nmi_status && self.ctrl.generate_nmi() && self.status.is_vblank() {
            self.nmi_ocurred = true;
        }
    }

    pub fn write_to_ppu_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_oam(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;

    }
    pub fn write_to_oam(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr += 1;
    }

    pub fn write_data(&mut self, value: u8) {
        let addr = self.addr.get() & 0x3FFF;
        // if !self.status.is_vblank() && self.mask.rendering() {
        //     self.addr.coarse_x_increment();
        //     self.addr.coarse_y_increment();
        // } else {
        //     self.increment_vram_addr();
        // }
        self.increment_vram_addr();
        match addr {
            0..=0x1FFF => {
                self.mapper.borrow_mut().write_chr(addr, value);
            },
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            },
            0x3000..=0x3EFF => {
                let addr = addr & 0x2EFF;
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            },
            0x3F00..=0x3FFF => {
                let mut addr = addr & 0x1F;
                if addr >= 0x10 && addr % 4 == 0 { 
                    addr -= 0x10; 
                }
                self.palette_table[addr as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", addr)
        }
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get() & 0x3FFF;
        // if !self.status.is_vblank() && self.mask.rendering() {
        //     self.addr.coarse_x_increment();
        //     self.addr.coarse_y_increment();
        // } else {
        //     self.increment_vram_addr();
        // }
        self.increment_vram_addr();
        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buff;
                self.internal_data_buff = self.mapper.borrow().read_chr(addr);
                result
            },
            0x2000..=0x2FFF => {
                let result = self.internal_data_buff;
                self.internal_data_buff = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            },
            0x3000..=0x3EFF => {
                let addr = addr & 0x2EFF;
                let result = self.internal_data_buff;
                self.internal_data_buff = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            },
            0x3F00..=0x3FFF => {
                self.internal_data_buff = self.vram[self.mirror_vram_addr(addr) as usize];
                let mut addr = addr & 0x1F;
                if addr >= 0x10 && addr % 4 == 0 { 
                    addr -= 0x10; 
                }
                self.palette_table[addr as usize]
            }
            _ => panic!("unexpected access to mirrored space {}", addr)
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        let mirroring = self.mapper.borrow().get_mirroring();
        // 00b - 1-screen mirroring (nametable 0)
        // 01b - 1-screen mirroring (nametable 1)
        match (mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            (Mirroring::OneScreenLower, 1) | (Mirroring::OneScreenLower, 2) | (Mirroring::OneScreenLower, 3) => vram_index & 0x23FF,
            (Mirroring::OneScreenUpper, 0) => vram_index + 0x400,
            (Mirroring::OneScreenUpper, 2) => vram_index - 0x400,
            (Mirroring::OneScreenUpper, 3) => vram_index - 0x800,
            _ => vram_index
        }
    }
}

