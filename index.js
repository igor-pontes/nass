const PIXEL_SIZE = 2;
const WIDTH = 256;
const HEIGHT = 240;

const imports = { 
  imports: { 
    log: (arg) => console.log(arg),
  } 
};

let buffer = new Uint8Array();
let wasm = { running: false };
let running = false;

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
    running = true;
    const fn = () => {
      drawCells(wasm.get_frame_pointer());
      drawPalettes(wasm.get_color);
      wasm.step();
      requestAnimationFrame(fn); 
    }
    requestAnimationFrame(fn);
  })
}

const getButton = (key) => {
    switch (key) {
      case "ArrowDown":
        return 0b00100000;
      case "ArrowUp":
        return 0b00010000;
      case "ArrowLeft":     
        return 0b01000000;
      case "ArrowRight":    
        return 0b10000000;
      case "Backspace":         
        return 0b00000100;
      case ".":          
        return 0b00000001;
      case ",":
        return 0b00000010;
      case "Escape":
        return 0b00001000;
      default:
        return 0;
    }
}

const toggleButton = (event) => {
  const button = getButton(event.key);
  if (button != 0 && running) 
    wasm.toggle_button(button);
}

document.addEventListener('keyup', toggleButton);

document.addEventListener('keydown', toggleButton);

drawCells(0);
drawPalettes(() => 0xFF);
