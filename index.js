import init, { disassemble, step, get_frame_pointer } from "./pkg/nass.js";

const COLORS  = [
    "#666666", "#002a88", "#1412a7", "#3b00a4", "#5c007e", "#6e0040", "#6c0600", "#561d00",
    "#333500", "#0b4800", "#005200", "#004f08", "#00404d", "#000000", "#000000", "#000000",
    "#adadad", "#155fd9", "#4240ff", "#7527fe", "#a01acc", "#b71e7b", "#b53120", "#994e00",
    "#6b6d00", "#388700", "#0c9300", "#008f32", "#007c8d", "#000000", "#000000", "#000000",
    "#fffeff", "#64b0ff", "#9290ff", "#c676ff", "#f36aff", "#fe6ecc", "#fe8170", "#ea9e22",
];

const PIXEL_SIZE = 8;
const WIDTH = 256;
const HEIGHT = 240;

const wasm = await init();
let buffer = new Uint8Array();
let frame_pointer;

const canvas = document.getElementById("nass-canvas");

canvas.width = WIDTH * 2;
canvas.height = HEIGHT * 2;

const ctx = canvas.getContext('2d');

document.getElementById("rom-input").onchange = getFile;

const drawCells = () => {
  for (let row = 0; row < HEIGHT; row++) {
    for (let col = 0; col < WIDTH; col++) {
      const color = COLORS[buffer[frame_pointer + HEIGHT * row + col]];
      ctx.fillStyle = color;
      ctx.fillRect(col * PIXEL_SIZE, row * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE);
      // ctx.fillRect(col, row, 1, 1);
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
  loadFile(file).then(rom => { 
    disassemble(rom);
    buffer = new Uint8Array(wasm.memory.buffer);
    frame_pointer = get_frame_pointer();
    const fn = () => {
      drawCells();
      step();
      requestAnimationFrame(fn); 
    }
    requestAnimationFrame(fn);
  })
}

drawCells();
