import init, { disassemble, step, get_frame_pointer, get_palette_pointer } from "./pkg/nass.js";

// https://en.wikipedia.org/wiki/List_of_video_game_console_palettes#Nintendo_Entertainment_System

const COLORS  = [
    "#59595f", "#00008f", "#18008f", "#3f0077", "#505", "#501", "#500", "#420", "#330", "#130", "#031", "#044", "#046", "#000", "#080808", "#080808",
    "#aaa", "#04d", "#51e", "#70e", "#90b", "#a05", "#930", "#840", "#660", "#360", "#060", "#065", "#058", "#080808", "#080808", "#080808",
    "#eee", "#48f", "#77f", "#94f", "#b4e", "#c59", "#d64", "#c80", "#ba0", "#7b0", "#2b2", "#2b7", "#2bc", "#444", "#080808", "#080808",
    "#eee", "#9cf", "#aaf", "#b9f", "#d9f", "#e9d", "#eaa", "#eb9", "#ed8", "#bd8", "#9d9", "#9db", "#9de", "#aaa", "#080808", "#080808",
];


const PIXEL_SIZE = 2;
const WIDTH = 256;
const HEIGHT = 240;
const wasm = await init();

const canvas = document.getElementById("nass-canvas");
canvas.width = WIDTH*2;
canvas.height = HEIGHT*2;
const ctx = canvas.getContext('2d');

const PALETTE_SIZE = 12;
const PALETTE_WIDTH = 16;
const palette_canvas = document.getElementById("palette-canvas");
palette_canvas.width = 16*PALETTE_SIZE;
palette_canvas.height = 2*PALETTE_SIZE;
const pctx = palette_canvas.getContext('2d');

let buffer = new Uint8Array();

document.getElementById("rom-input").onchange = getFile;

const drawCells = (pointer) => {
  for (let row = 0; row < HEIGHT; row++) {
    for (let col = 0; col < WIDTH; col++) {
      const color = COLORS[buffer[pointer + row * PALETTE_WIDTH + col]];
      ctx.fillStyle = color;
      ctx.fillRect(col * PIXEL_SIZE, row * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE);
    }
  }
}

const drawPalettes = (pointer) => {
  for (let row = 0; row < 32/PALETTE_WIDTH; row++) {
    for (let col = 0; col < PALETTE_WIDTH; col++) {
      const color = COLORS[buffer[pointer + row * PALETTE_WIDTH + col]];
      pctx.fillStyle = color;
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
  loadFile(file).then(rom => { 
    disassemble(rom);
    buffer = new Uint8Array(wasm.memory.buffer);
    const fn = () => {
      drawCells(get_frame_pointer());
      drawPalettes(get_palette_pointer());
      step();
      requestAnimationFrame(fn); 
    }
    requestAnimationFrame(fn);
  })
}

drawCells(0);
drawPalettes(0);
