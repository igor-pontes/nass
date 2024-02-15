const PIXEL_SIZE = 2;
const WIDTH = 256;
const HEIGHT = 240;

const imports = { 
  imports: { 
    log: (arg) => console.log(arg),
  } 
};

let buffer = new Uint8Array();
let wasm = {};

WebAssembly.instantiateStreaming(fetch('target/wasm32-unknown-unknown/release/nass.wasm'), imports).then(obj => { wasm = obj.instance.exports; });

const canvas = document.getElementById("nass-canvas");
canvas.width = WIDTH*2;
canvas.height = HEIGHT*2;
const ctx = canvas.getContext('2d');

const PALETTE_SIZE = 12;
const PALETTE_WIDTH = 16;
const palette_canvas = document.getElementById("palette-canvas");
palette_canvas.height = 2*PALETTE_SIZE;
const pctx = palette_canvas.getContext('2d');
palette_canvas.width = 16*PALETTE_SIZE;

document.getElementById("rom-input").onchange = getFile;

const getRgba = (r, g, b, a) => `rgba(${r}, ${g}, ${b}, ${a})`;

const drawCells = (pointer) => {
  for (let row = 0; row < HEIGHT; row++) {
    for (let col = 0; col < WIDTH; col++) {
        const offset = pointer + row * WIDTH*4 + col*4;
        const red = buffer[offset + 3];
        const green = buffer[offset + 2];
        const blue = buffer[offset + 1];
        const alpha = buffer[offset];
        ctx.fillStyle = getRgba(red, green, blue, alpha);
        ctx.fillRect(col * PIXEL_SIZE, row * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE);
    }
  }
}

const drawPalettes = (getColor) => {
  for (let row = 0; row < 32/PALETTE_WIDTH; row++) {
    for (let col = 0; col < PALETTE_WIDTH; col++) {
      let index = row * PALETTE_WIDTH + col;
      if (index >= 0x10 && index % 4 == 0) { index -= 0x10; }
      const color = getColor(index);
      const red = (color & 0xFF000000) >>> 24;
      const green = (color & 0x00FF0000) >>> 16;
      const blue = (color & 0x0000FF00) >>> 8;
      const alpha = (color & 0x000000FF);
      pctx.fillStyle = getRgba(red, green, blue, alpha);
      pctx.fillRect(col * PALETTE_SIZE, row * PALETTE_SIZE, PALETTE_SIZE, PALETTE_SIZE);
    }
  }
}

function getFile() {
  const file = document.getElementById("rom-input").files[0];
  const loadFile = (file) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => resolve(e.target.result);
      reader.onerror = (_) => reject(reader.error);
      reader.readAsArrayBuffer(file);
    })
  }
  loadFile(file).then(rom_buffer => { 
    const rom = new Uint8Array(rom_buffer);
    wasm.set_rom_length(rom.length);
    buffer = new Uint8Array(wasm.memory.buffer);
    buffer.set(rom, wasm.get_rom_pointer())
    wasm.disassemble();
    wasm.reset();
    buffer = new Uint8Array(wasm.memory.buffer);
    const fn = () => {
      drawCells(wasm.get_frame_pointer());
      drawPalettes(wasm.get_color);
      wasm.step();
      requestAnimationFrame(fn); 
    }
    requestAnimationFrame(fn);
  })
}

// Handle inputs
addEventListener(
  "keydown",
  (event) => {
    if (event.defaultPrevented) { return; }
    switch (event.key) {
      case "ArrowDown":
        wasm.set_button(0b00100000);
        break;
      case "ArrowUp":
        wasm.set_button(0b00010000);
        break;               
      case "ArrowLeft":      
        wasm.set_button(0b01000000);
        break;               
      case "ArrowRight":     
        wasm.set_button(0b10000000);
        break;               
      case "Backspace":          
        wasm.set_button(0b00000100);
        break;
      case ".":          
        wasm.set_button(0b00000001);
        break;
      case ",":
        wasm.set_button(0b00000010);
        break;
      case "Escape":
        wasm.set_button(0b00001000);
        break;
      default:
        return;
    }
    event.preventDefault();
  },
  true,
);

drawCells(0);
drawPalettes(() => 0xFF);
