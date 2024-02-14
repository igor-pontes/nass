mod ppu_addr;
mod ppu_control;
mod ppu_mask;
mod ppu_status;
mod colors;
mod line;

pub use colors::*;
use line::{*, Line::*};
use crate::frame::Frame;

use crate::mapper::*;
use self::{
    ppu_addr::PPUAddr,
    ppu_control::PPUControl,
    ppu_mask::PPUMask,
    ppu_status::PPUStatus,
};

pub struct PPU {
    pub palette_table: [u8; 0x20],
    vram: [u8; 0x800], // Nametables (2kB)
    oam_data: [u8; 0x100],
    sprites: ([u8; 0x20], usize),
    pub oam_addr: u8,
    addr: PPUAddr,
    temp: u16,
    ctrl: PPUControl,
    pub mask: PPUMask,
    status: PPUStatus,
    internal_data_buff: u8,
    // fine_x: u8,
    line: Line,
    dot: usize,
    pub frame: Frame,
    pub nmi_occured: bool
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            palette_table: [0; 0x20],
            vram: [0; 0x800],
            oam_data: [0; 0x100],
            sprites: ([0; 0x20], 0),
            oam_addr: 0,
            addr: PPUAddr::new(),
            ctrl: PPUControl::new(),
            temp: 0,
            mask: PPUMask::new(),
            status: PPUStatus::new(),
            internal_data_buff: 0,
            // fine_x: 0,
            line: Render(0),
            dot: 0,
            frame: Frame::new(),
            nmi_occured: false
        }
    }

    pub fn tick(&mut self, mapper: &mut Mapper_) {
        match self.line {
            PreRender => {
                if self.dot == 1 { self.status.reset(); }
                if self.mask.rendering() && self.dot > 0 {
                    if self.dot % 8 == 0 && self.dot <= 256 { self.addr.coarse_x_increment(); } 
                    if self.dot == 256 { self.addr.coarse_y_increment(); }
                    if self.dot == 257 { self.oam_addr = 0; self.addr.set_horizontal(self.temp); }
                    if self.dot >= 280 && self.dot <= 304 { self.addr.set_vertical(self.temp); }
                }
            },
            Render(_) => {
                if self.dot > 0 {
                    if self.dot <= 256 {
                        let mut color = 0;
                        if self.mask.show_background() && (self.dot > 8 || self.mask.show_background_leftmost()) {
                            let v = self.addr.get();
                            let fine_x = 8 - (self.dot as u8) % 8;
                            let fine_y = (v & 0x7000) >> 12;

                            let tile_addr = 0x2000 | (v & 0x0FFF);
                            let tile = self.vram[mapper.mirror(tile_addr) as usize];
                            let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                            let attr_data = self.vram[mapper.mirror(attr_addr) as usize];

                            let half_pattern_table = self.ctrl.get_background_pattern_addr();
                            let color_addr_1 = half_pattern_table | (tile as u16) << 4 | 1 << 3 | fine_y;
                            let color_addr_0 = half_pattern_table | (tile as u16) << 4 | 0 << 3 | fine_y;
                            let color_bit_0 = ( mapper.read_chr(color_addr_0) >> fine_x) & 0x1;
                            let color_bit_1 = ( mapper.read_chr(color_addr_1) >> fine_x) & 0x1;
                            let color_tile = (color_bit_1 << 1) | color_bit_0;

                            let tile_column = (v & 0x001f) as u8;
                            let tile_row = ((v & 0x03e0) >> 5) as u8;
                            let quadrant = (tile_row & 0x2) + ((tile_column & 0x2) >> 1);
                            let offset = quadrant * 2;
                            let attr_color = (attr_data >> offset) & 0x03;
                            if color_tile > 0 { color = (attr_color << 2 | color_tile) as usize; }
                        }
                        
                        if self.mask.show_sprite() && (self.dot > 8 || self.mask.show_sprite_leftmost()) {
                            for sprite in 0..self.sprites.1 {
                                let x = self.sprites.0[4*sprite + 3] as usize;
                                if (self.dot - x) & 0xFF < 8 {
                                    let y = self.sprites.0[4*sprite] as usize;
                                    let tile = self.sprites.0[4*sprite + 1] as u16;
                                    let attr = self.sprites.0[4*sprite + 2];
                                    let bank = (tile & 0x1) << 12;
                                    let priority = attr & 0x2 == 0;
                                    let palette = attr & 0x03;
                                    let flip_h = attr & 0x40 > 0;
                                    let flip_v = attr & 0x80 > 0;
                                    let height = if self.ctrl.is_sprite_size_16() { 16 } else { 8 };
                                    let y = self.line.get() - y - 1;
                                    let x = self.dot - x;
                                    let fine_x = if flip_h { x } else { 7 - x };
                                    let fine_y = if flip_v { height - 1 - y } else { y } as u16;
                                    let offset = y.div_euclid(8) as u16;
                                    let half_pattern_table = if self.ctrl.is_sprite_size_16() { bank } else { self.ctrl.get_sprite_pattern_addr()};
                                    let color_addr_0 = half_pattern_table | tile << 4 | 0 << 3 | fine_y;
                                    let color_bit_0 = ( mapper.read_chr(color_addr_0) >> fine_x) & 0x1;
                                    let color_addr_1 = half_pattern_table | tile + offset << 4 | 1 << 3 | fine_y;
                                    let color_bit_1 = ( mapper.read_chr(color_addr_1) >> fine_x) & 0x1;
                                    let color_tile = (color_bit_1 << 1) | color_bit_0;

                                    if color_tile > 0 { 
                                        if priority || color == 0 { 
                                            if !self.status.sprite_hit() && self.mask.show_background() { self.status.set_sprite_hit(true); }
                                            color = (0x10 | palette << 2 | color_tile) as usize 
                                        } 
                                    }
                                }
                            }
                        }
                        if self.mask.rendering() {
                            self.frame.set_pixel(COLORS[self.palette_table[color] as usize]);
                        }
                    }

                    if self.mask.rendering() {
                        if self.dot % 8 == 0 && self.dot <= 256 { self.addr.coarse_x_increment(); } 
                        if self.dot == 256 { self.addr.coarse_y_increment(); }
                        if self.dot == 257 { self.oam_addr = 0; self.addr.set_horizontal(self.temp); }
                        if self.dot == 270 {
                            self.sprites = ([0; 0x20], 0);
                            let height = if self.ctrl.is_sprite_size_16() { 16 } else { 8 };
                            for n in (0..self.oam_data.len()).step_by(4) {
                                let y = self.oam_data[n] as usize;
                                if (self.line.get() - y) < height && (239 - y) >= height {
                                    if self.sprites.1 < 8{
                                        self.sprites.0[4*self.sprites.1] = self.oam_data[n];
                                        self.sprites.0[4*self.sprites.1 + 1] = self.oam_data[n + 1];
                                        self.sprites.0[4*self.sprites.1 + 2] = self.oam_data[n + 2];
                                        self.sprites.0[4*self.sprites.1 + 3] = self.oam_data[n + 3];
                                        self.sprites.1 += 1;
                                    } else {
                                        self.status.set_overflow(true);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            PostRender(line) => {
                if line == 241 && self.dot == 1 {
                    self.status.set_vblank(true);
                    if self.ctrl.generate_nmi() { 
                        self.nmi_occured = true; 
                    }
                }
            },
        }
        self.line.next(&mut self.dot);
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        if !self.addr.latch() {
            // self.fine_x = value & 0x7;
            let value = value >> 3;
            self.temp = (self.temp & 0xFFE0) | (value as u16);
        } else {
            let fine_y_scroll = (value & 0x07) as u16;
            let coarse_y_scroll = (value & 0xF8) as u16;
            self.temp = (self.temp & 0x8C1F) | fine_y_scroll << 12 | coarse_y_scroll << 2;
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

    pub fn write_to_ctrl(&mut self, value: u8) -> bool {
        let before_nmi_status = self.ctrl.generate_nmi();
        self.ctrl.update(value, &mut self.temp);
        if !before_nmi_status && self.ctrl.generate_nmi() && self.status.is_vblank() {
            return true
        }
        false
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_oam(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr += 1;
    }

    pub fn write_data(&mut self, value: u8, mapper: &mut Mapper_) {
        let addr = self.addr.get() & 0x3FFF;
        self.increment_vram_addr();
        match addr {
            0..=0x1FFF => mapper.write_chr(addr, value),
            0x2000..=0x2FFF => self.vram[mapper.mirror(addr) as usize] = value,
            0x3000..=0x3EFF => self.vram[mapper.mirror(addr-0x1000) as usize] = value,
            0x3F00..=0x3FFF => {
                let mut addr = (addr & 0x1F) as u8;
                if addr >= 0x10 && addr % 4 == 0 { 
                    addr -= 0x10; 
                }
                self.palette_table[addr as usize] = value;
            }
            _ => panic!("Unexpected access to mirrored space {}", addr)
        }
    }

    pub fn read_data(&mut self, mapper: &Mapper_) -> u8 {
        let addr = self.addr.get() & 0x3FFF;
        self.increment_vram_addr();
        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buff;
                self.internal_data_buff = mapper.read_chr(addr);
                result
            },
            0x2000..=0x2FFF => {
                let result = self.internal_data_buff;
                self.internal_data_buff = self.vram[mapper.mirror(addr) as usize];
                result
            },
            0x3000..=0x3EFF => {
                let result = self.internal_data_buff;
                self.internal_data_buff = self.vram[mapper.mirror(addr - 0x1000) as usize];
                result
            },
            0x3F00..=0x3FFF => {
                self.internal_data_buff = self.vram[mapper.mirror(addr) as usize];
                let mut addr = addr & 0x1F;
                if addr >= 0x10 && addr % 4 == 0 { 
                    addr -= 0x10; 
                }
                self.palette_table[addr as usize]
            }
            _ => panic!("Unexpected access to mirrored space {}", addr)
        }
    }
}

