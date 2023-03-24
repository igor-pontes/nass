import { Scene, run } from "./pkgs/nass_bg.js";
//import { Scene, disassemble, run } from "nass_bg";
const CELL_SIZE = 1;
const GRID_COLOR = "#CCCCCC";
const ALIVE_COLOR = "#000000";

const scene = Scene.new();
const width = scene.width;
const height = scene.height;

const canvas = document.getElementById("nass-canvas");
canvas.height = CELL_SIZE * height;
canvas.width = CELL_SIZE * width;

const ctx = canvas.getContext('2d');

const renderLoop = () => {
  //universe.tick();
  run();
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

      if (scene.pixels) {
        ctx.fillStyle = scene.pixels[row][col];
      } else {
        ctx.fillStyle = "black";
      }

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

let element = document.getElementById("rom-input");
element.onchange = getFile;

function getFile() {
  var file = element.files[0];
  const reader = new FileReader();
  reader.readAsDataURL(file);
  reader.addEventListener('load', () => {
    localStorage.setItem('file', reader.result
        .replace('data:', '')
        .replace(/^.+,/, ''));
  });
  //disassemble(localStorage.getItem("file"), scene);
}

drawGrid();
drawCells();
requestAnimationFrame(renderLoop);
