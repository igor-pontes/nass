import init, { disassemble, step } from "./pkg/nass.js";

const CELL_SIZE = 1;
const GRID_COLOR = "#CCCCCC";
const ALIVE_COLOR = "#000000";

await init();

const canvas = document.getElementById("nass-canvas");
const width = 10;
const height = 10;
canvas.height = CELL_SIZE * width;
canvas.width = CELL_SIZE * height;

const ctx = canvas.getContext('2d');

const renderLoop = () => {
  step(); // Emulator step
  drawGrid();
  drawCells();
  requestAnimationFrame(renderLoop);
};

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i < width; i++) {
    ctx.moveTo(i * CELL_SIZE, 0);
    ctx.lineTo(i * CELL_SIZE, CELL_SIZE * height);
  }

  // Horizontal lines.
  for (let j = 0; j < height; j++) {
    ctx.moveTo(0, j * CELL_SIZE);
    ctx.lineTo(CELL_SIZE * width, j * CELL_SIZE);
  }

  ctx.stroke();
};

const drawCells = () => {
  ctx.beginPath();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {

      ctx.fillStyle = "black";

      ctx.fillRect(
        col * CELL_SIZE,
        row * CELL_SIZE,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }
  ctx.stroke();
};

document.getElementById("rom-input").onchange = getFile;

function getFile() {
  var file = element.files[0];
  const loadFile = (file) => {
    return Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => resolve(e.target.result);
      reader.onerror = (e) => reject(reader.error);
      reader.readAsArrayBuffer(file);
    })
  }
  disassemble(loadFile(file));
}

drawGrid();
drawCells();
requestAnimationFrame(renderLoop);
