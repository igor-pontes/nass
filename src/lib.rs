#![feature(bigint_helper_methods)]

mod ppu;
mod cpu;
mod emulator;
mod mapper;
mod frame;

use { 
    cfg_if::cfg_if,
    std::cell::RefCell,
    crate::emulator::Emulator,
};

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

thread_local!{ static EMULATOR: RefCell<Emulator> = RefCell::new(Emulator::new()) }

#[no_mangle]
pub fn set_rom_length(value: usize) {
    EMULATOR.with_borrow_mut(|e| e.set_len(value))
}

#[no_mangle]
pub fn disassemble() {
    EMULATOR.with_borrow_mut(|e| e.disassemble());
}

#[no_mangle]
pub fn reset() {
    EMULATOR.with_borrow_mut(|e| e.reset());
}

#[no_mangle]
pub fn step() {
    EMULATOR.with_borrow_mut(|e| e.step());
}

#[no_mangle]
pub fn set_button(value: u8) {
    EMULATOR.with_borrow_mut(|e| e.set_button(value));
}

#[no_mangle]
pub fn get_frame_pointer() -> *const u32 {
    EMULATOR.with_borrow_mut(|e| e.get_frame_pointer())
}

#[no_mangle]
pub fn get_rom_pointer() -> *const u8 {
    EMULATOR.with_borrow_mut(|e| e.get_rom_pointer())
}

#[no_mangle]
pub fn get_color(index: usize) -> u32 {
    EMULATOR.with_borrow_mut(|e| e.get_color(index))
}
