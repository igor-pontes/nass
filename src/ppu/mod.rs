use std::rc::Rc;
use std::cell::RefCell;
mod color;
mod addr_register;
mod control_register;
mod ppu_mask;
mod ppu_status;
pub use self::color::*;
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
    // OAM
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
    frame: Frame
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>> ) -> Self {
        PPU {
            mapper,
            palette_table: [0; 0x20],
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
            scanline: 261,
            cycle: 0,
            odd_frame: false, // "true" because we switch to "false" in frame 0.
            frame: Frame::new()
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt )  {
        // log(&format!("[PPU] dot: {} | line: {}", self.cycle, self.scanline));
        // log(&format!("[PPU] line: {}", self.scanline));
        let mut color_bg = 0;
        match self.scanline {
            261 => { // Pre-render
                if self.cycle == 1 { 
                    self.status.update(self.status.bits() & 0x7F);
                    (*interrupt) = Interrupt::DISABLED; 
                }
                if (self.cycle == 339 && self.odd_frame) || self.cycle == 340 { 
                    self.scanline = 0; self.cycle = 0; 
                    self.odd_frame = !self.odd_frame;
                }
            },
            0..=239 => { // Render
                if self.mask.show_background()   {
                    log("------- Background enabled. -------");
                    let show_leftmost = (((self.mask.show_background_leftmost() as u8) ^ (self.cycle <= 8) as u8)) != 0;
                    if self.cycle != 0 && !show_leftmost || self.cycle > 8 {
                        let v = self.addr.get();
                        let fine_y = v & 0x7000 >> 12;

                        let tile_addr = 0x2000 | (v & 0x0FFF);
                        let tile = self.vram[(tile_addr - 0x2000) as usize];

                        // https://www.nesdev.org/wiki/PPU_attribute_tables
                        let attr_addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                        let attr_data = self.vram[(attr_addr - 0x2000) as usize];

                        let half_pattern_table = if self.ctrl.get_background_pattern_addr() { 0x1000 } else { 0 };
                        let color_addr_0 = half_pattern_table | (tile as u16) << 4 | 1 << 3 | fine_y;
                        let color_addr_1 = half_pattern_table | (tile as u16) << 4 | 0 << 3 | fine_y;
                        let color_bit_0 = unsafe { ( (*self.mapper.as_ptr()).read_chr(color_addr_0) >> self.fine_x ) & 0x1 };
                        let color_bit_1 = unsafe { ( ( (*self.mapper.as_ptr()).read_chr(color_addr_1) >> self.fine_x ) & 0x1 ) << 1 };
                        let color_tile = color_bit_1 | color_bit_0;

                        let tile_column = (v & 0x1f) as u8;
                        let tile_row = ((v & 0x3e0) >> 5) as u8;
                        let quadrant = (tile_row & 0x2) + ((tile_column & 0x2) >> 1);
                        let attr_color = (attr_data & (0x3 << (quadrant * 2))) >> (quadrant * 2);
                        log(&format!("attr_color: {}; color_tile: {}", attr_color, color_tile));
                        color_bg = (0x10 | attr_color | color_tile) as u8;

                        log(&format!("PPU color: {}", color_bg));

                        // Increment address
                        if self.cycle % 8 == 0 {
                           if self.cycle == 256 {
                               self.addr.y_increment();
                               self.addr.coarse_x_increment();
                           } else {
                               self.addr.coarse_x_increment();
                           }
                        }
                    }
                }

                if self.mask.show_sprite() {
                    let show_leftmost = (self.mask.show_sprite_leftmost() as u8 ^ (self.cycle <= 8) as u8) != 0;
                    if self.cycle != 0 && !show_leftmost || self.cycle > 8 {
                        // TODO
                    }
                }

                // Set Frame's pixel
                // log(&color::to_hex(color_bg & 0xFF));
                // self.frame.set_pixel(0, 0, color::COLORS[(color_bg & 0xFF) as usize])
            }
            240 => {
                // Post-render
                // self.frame.set_pixel(0, 0, color::COLORS[(color_bg & 0xFF) as usize])
            }
            241..=u16::MAX => {
                // Vertical Blank Lines
                if self.scanline == 241 && self.cycle == 1 { 
                    self.status.update(self.status.bits() | 0x80);
                    if self.ctrl.generate_nmi() {
                        // log("----- NMI -----");
                        // (*interrupt) = Interrupt::NMI; 
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

    pub fn copy_to_oam(&mut self, arr: &[u8; 0x100]) {
        self.oam_data = arr.clone();
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

