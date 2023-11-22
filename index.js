import init, { disassemble } from "./pkg/nass.js";

await init();

const canvas = document.getElementById("nass-canvas");
const ctx = canvas.getContext('2d');

const height = 256;
const width = 240;

canvas.height = 256;
canvas.width = 240;

document.getElementById("rom-input").onchange = getFile;

const drawCells = () => {
  ctx.beginPath();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {

      ctx.fillStyle = "#000000";

      ctx.fillRect(
        col,
        row,
        1,
        1
      );
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
    const step = disassemble(rom);
    const fn = () => {
      step();
      drawCells();
      requestAnimationFrame(fn); 
    }
    requestAnimationFrame(fn);
  })
}

drawCells();
