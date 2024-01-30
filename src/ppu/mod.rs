use std::rc::Rc;
use std::cell::RefCell;
mod addr_register;
mod control_register;
mod ppu_mask;
mod ppu_status;
use crate::frame::Frame;
use crate::Interrupt;
use crate::mapper::*;
pub use self::addr_register::AddrRegister;
pub use self::control_register::ControlRegister;
pub use self::ppu_mask::PPUMask;
pub use self::ppu_status::PPUStatus;
use crate::mapper::Mirroring;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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
    // x: usize,
    // y: usize,
    // x_fine: usize,
    // y_fine: u16,
    // pub debug: bool
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>> ) -> Self {
        PPU {
            palette_table: [0; 0x20],
            mapper,
            vram: [0; 0x800],
            oam_data: [0; 0x100],
            oam_addr: 0x0,
            addr: AddrRegister::new(),
            temp: 0x0,
            ctrl: ControlRegister::new(),
            mask: PPUMask::new(),
            status: PPUStatus::new(),
            internal_data_buff: 0x0,
            fine_x: 0x0,
            scanline: 261,
            cycle: 0x0,
            odd_frame: false, // "true" because we switch to "false" in frame 0.
            frame: Frame::new(),
            // x: 0,
            // y: 0,
            // x_fine: 0,
            // y_fine: 0,
            // debug: false,
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt )  {
        // log(&format!("[PPU] dot: {} | line: {}", self.cycle, self.scanline));
        // log(&format!("[PPU] line: {}", self.scanline));
        let mut color_bg = 0;
        match self.scanline {
            261 => { // Pre-render
                if self.cycle > 0 {
                    if self.cycle == 1 { 
                        self.status.update(self.status.bits() & 0x7F);
                    }
                    if (self.cycle == 339 && self.odd_frame) || self.cycle == 340 { 
                        self.scanline = 0; self.cycle = 0; 
                        self.odd_frame = !self.odd_frame;
                    }
                    // if self.cycle % 8 == 0 && (self.cycle <= 256 || self.cycle >= 328) {
                    //     self.addr.coarse_x_increment();
                    //     if self.cycle == 256 { self.addr.y_increment(); }
                    // }
                    
                    if self.mask.show_background() { 
                        if self.cycle % 8 == 0 && (self.cycle <= 256 || self.cycle >= 328 ){
                            self.addr.coarse_x_increment();
                            if self.cycle == 256 { self.addr.y_increment(); }
                        }
                        if self.cycle == 257 { self.addr.set_horizontal(self.temp); }
                        if  self.cycle >= 280 || self.cycle <= 304 { self.addr.set_vertical(self.temp); }
                    }
                }
            },
            0..=239 => { // Render
                if self.cycle > 0 {

                    if self.mask.show_background() {
                        // log("------- Background enabled. -------");
                        // let show_leftmost = (((self.mask.show_background_leftmost() as u8) ^ (self.cycle <= 8) as u8)) != 0;
                        // if (self.cycle != 0 && !show_leftmost) || self.cycle > 8 {
                            // In each 8-dot window, the PPU performs the 4 memory fetches required to produce 8 pixels
                            let v = self.addr.get();
                            let fine_y = v & 0x7000 >> 12;

                            let tile_addr = 0x2000 | (v & 0x0FFF);
                            let tile = self.vram[self.mirror_vram_addr(tile_addr) as usize];

                            // https://www.nesdev.org/wiki/PPU_attribute_tables
                            let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                            let attr_data = self.vram[self.mirror_vram_addr(attr_addr) as usize];

                            let half_pattern_table = if self.ctrl.get_background_pattern_addr() { 0x1000 } else { 0 };
                            let color_addr_0 = half_pattern_table | (tile as u16) << 4 | 0 << 3 | fine_y;
                            let color_addr_1 = half_pattern_table | (tile as u16) << 4 | 1 << 3 | fine_y;
                            let color_bit_0 = ( self.mapper.borrow().read_chr(color_addr_0) >> self.fine_x ) & 0x1;
                            let color_bit_1 = ( ( self.mapper.borrow().read_chr(color_addr_1) >> self.fine_x ) & 0x1 ) << 1;
                            let color_tile = color_bit_1 | color_bit_0;

                            let tile_column = (v & 0x1f) as u8;
                            let tile_row = ((v & 0x3e0) >> 5) as u8;
                            let quadrant = (tile_row & 0x2) + ((tile_column & 0x2) >> 1);
                            let attr_color = (attr_data & (0x3 << (quadrant * 2))) >> (quadrant * 2);
                            // color_bg = (0x10 | attr_color | color_tile) as u8;
                            color_bg = self.palette_table[( 0x10 | attr_color << 2 | color_tile) as usize];

                            // Increment address
                            // if self.cycle % 8 == 0 && (self.cycle <= 256 || self.cycle >= 328) {
                            // log(&format!("No problems here."));
                            if self.cycle % 8 == 0 && (self.cycle <= 256 || self.cycle >= 328 ){
                                self.addr.coarse_x_increment();
                                if self.cycle == 256 { self.addr.y_increment(); }
                            }
                            if self.cycle == 257 { 
                                self.addr.set_horizontal(self.temp); 
                            }
                        // }
                    }

                    if self.cycle <= 256 {
                        self.frame.set_pixel(color_bg);
                    }

                    // if self.mask.show_sprite() {
                    //     let show_leftmost = (self.mask.show_sprite_leftmost() as u8 ^ (self.cycle <= 8) as u8) != 0;
                    //     if (self.cycle != 0 && !show_leftmost) || self.cycle > 8 {
                    //         // TODO
                    //     }
                    // }
                    // if self.cycle <= 256 && !self.debug {
                    //     // Debug.
                    //     // Each nametable has 30 rows of 32 tiles each, for 960 ($3C0) bytes; the rest is used by each nametable's attribute table. 
                    //     let y = self.y & 0x1F;
                    //     let x = self.x & 0x1F;
                    //     // let v = self.y_fine | 0x0C00 | ((y << 5) | x) as u16;
                    //     let v = self.y_fine | 0x0000 | ((y << 5) | x) as u16;
                    //
                    //     let tile = self.vram[self.mirror_vram_addr(0x2000 | (v & 0x0FFF)) as usize];
                    //
                    //     //  NN 1111 YYY XXX
                    //     let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                    //     let attr_data = self.vram[self.mirror_vram_addr(attr_addr) as usize];
                    //
                    //     let half_pattern_table = if self.ctrl.get_background_pattern_addr() { 0x1000 } else { 0 };
                    //     let color_addr_0 = half_pattern_table | (tile as u16) << 4 | 0 << 3 | (self.y_fine >> 12);
                    //     let color_addr_1 = half_pattern_table | (tile as u16) << 4 | 1 << 3 | (self.y_fine >> 12);
                    //     let color_bit_0 = ( self.mapper.borrow().read_chr(color_addr_0) >> self.x_fine ) & 0x1;
                    //     let color_bit_1 = (( self.mapper.borrow().read_chr(color_addr_1) >> self.x_fine ) & 0x1 ) << 1 ;
                    //     let color_tile = color_bit_1 | color_bit_0;
                    //
                    //     let tile_column = (v & 0x1f) as u8;
                    //     let tile_row = ((v & 0x3e0) >> 5) as u8;
                    //     let quadrant = (tile_row & 0x2) + ((tile_column & 0x2) >> 1);
                    //     let attr_color = (attr_data & (0x3 << (quadrant * 2))) >> (quadrant * 2);
                    //     color_bg = self.palette_table[(0x10 | attr_color << 2 | color_tile) as usize];
                    //
                    //     if self.cycle % 8 == 0 {
                    //         self.x_fine = 8; 
                    //         if self.x == 31 {
                    //             self.x = 0;
                    //         } else {
                    //             self.x += 1;
                    //         }
                    //         if self.cycle == 256 {
                    //             if self.y_fine != 0x7000 {
                    //                 self.y_fine += 0x1000;
                    //             } else {
                    //                 self.y_fine = 0x0000;
                    //                 if self.y > 31 {
                    //                     self.y = 0;
                    //                 } else {
                    //                     self.y += 1; // increment coarse
                    //                 }
                    //             }
                    //         }
                    //     }
                    //     self.x_fine -= 1; 
                    //     self.frame.set_pixel(color_bg);
                    // }
                }
            }
            240 => {
                // Post-render
                // log(&format!("X: {} | Y: {}", self.x, self.y));
                // self.y = 0;
                self.frame.set_frame();
            }
            241..=u16::MAX => {
                // Vertical Blank Lines
                if self.scanline == 241 && self.cycle == 1 { 
                    self.status.update(self.status.bits() | 0x80);
                    if self.ctrl.generate_nmi() {
                        (*interrupt) = Interrupt::NMI; 
                    }
                }
            }
        }
        self.cycle += 1;
        if self.cycle == 341 { self.scanline += 1; self.cycle = 0; }
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        if self.addr.latch() {
            self.fine_x = value & 0b00000111;
            let value = (value & 0b11111000) >> 3;
            self.temp = self.temp & 0b111111111100000 | value as u16;
        } else {
            let fine_y_scroll = value & 0b00000111;
            let coarse_y_scroll = value & 0b11111000;
            self.temp = self.temp & 0b111001111100000 | ( fine_y_scroll as u16 ) << 12 | (coarse_y_scroll as u16) << 2;
        }
        self.addr.toggle_latch();
    }

    pub fn status(&mut self) -> u8 {
        let status = self.status.bits();
        self.addr.reset_latch();
        status
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value, &mut self.temp);
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        self.ctrl.update(value, &mut self.temp);
    }

    pub fn write_to_ppu_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }
    
    pub fn write_to_oam(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr += 1;
    }

    pub fn write_data(&mut self, value: u8) {
        // log(&format!("PPU<write_data()> VRAM_ADDR({:#06x})", self.addr.get()));
        let addr = self.addr.get();
        self.increment_vram_addr();
        match addr {
            0..=0x1FFF => {
                self.mapper.borrow_mut().write_chr(addr, value);
            },
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            },
            0x3000..=0x3EFF => {
                let addr = addr - 0x1000;
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            },
            0x3F00..=0x3FFF => {
                self.palette_table[(addr & 0x1F) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", addr)
        }
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
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
                let addr = addr - 0x1000;
                let result = self.internal_data_buff;
                self.internal_data_buff = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            },
            0x3F00..=0x3FFF => {
                self.palette_table[(addr & 0x1F) as usize]
            }
            _ => panic!("unexpected access to mirrored space {}", addr)
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        let mirroring = unsafe { (*self.mapper.as_ptr()).get_mirroring() };
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

