use std::fmt;

#[wasm_bindgen]
pub struct Scene {
    width: u32,
    height: u32,
    pixels: Option<u8>,
}

#[wasm_bindgen]
impl Scene {
    fn new() -> Scene {
        Scene {
            width: 256,
            height: 240,
            pixels: None,
        }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
}

impl fmt::Display for Scene {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                write!(f, "{x}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}