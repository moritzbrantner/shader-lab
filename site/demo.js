import { presetById, presets } from "./shaders.js";
import { requestDevice } from "./webgpu.js";

const canvas = document.querySelector("#shader-canvas");
const editor = document.querySelector("#shader-source");
const presetSelect = document.querySelector("#preset");
const compileButton = document.querySelector("#compile");
const pauseButton = document.querySelector("#pause");
const resetButton = document.querySelector("#reset");
const speedInput = document.querySelector("#speed");
const question = document.querySelector("#experiment-question");
const status = document.querySelector("#compile-status");
const diagnostics = document.querySelector("#diagnostics");
const frameMetric = document.querySelector("#frame-metric");
const resolutionMetric = document.querySelector("#resolution-metric");
const adapterMetric = document.querySelector("#adapter-metric");

if (
  !(canvas instanceof HTMLCanvasElement) ||
  !(editor instanceof HTMLTextAreaElement) ||
  !(presetSelect instanceof HTMLSelectElement) ||
  !(compileButton instanceof HTMLButtonElement) ||
  !(pauseButton instanceof HTMLButtonElement) ||
  !(resetButton instanceof HTMLButtonElement) ||
  !(speedInput instanceof HTMLInputElement) ||
  !(question instanceof HTMLElement) ||
  !(status instanceof HTMLElement) ||
  !(diagnostics instanceof HTMLElement) ||
  !(frameMetric instanceof HTMLElement) ||
  !(resolutionMetric instanceof HTMLElement) ||
  !(adapterMetric instanceof HTMLElement)
) {
  throw new Error("Shader Lab experiment markup is incomplete.");
}

for (const preset of presets) {
  const option = document.createElement("option");
  option.value = preset.id;
  option.textContent = preset.label;
  presetSelect.append(option);
}

let selectedPreset = presets[0];
let device = null;
let context = null;
let format = null;
let uniformBuffer = null;
let bindGroupLayout = null;
let bindGroup = null;
let pipelineLayout = null;
let pipeline = null;
let animationFrame = 0;
let paused = false;
let startTime = performance.now();
let pointer = [0.5, 0.5];
let frameSamples = [];

function setPreset(preset) {
  selectedPreset = preset;
  presetSelect.value = preset.id;
  editor.value = preset.source.trimStart();
  question.textContent = preset.question;
}

function setStatus(kind, message) {
  status.dataset.state = kind;
  status.textContent = message;
}

function renderDiagnostics(messages) {
  diagnostics.replaceChildren();
  if (messages.length === 0) {
    const item = document.createElement("li");
    item.textContent = "WGSL compiled without diagnostics.";
    diagnostics.append(item);
    return;
  }

  for (const message of messages) {
    const item = document.createElement("li");
    item.dataset.type = message.type;
    const location = message.lineNum > 0 ? `line ${message.lineNum}:${message.linePos}` : "shader";
    item.textContent = `${message.type.toUpperCase()} · ${location} · ${message.message}`;
    diagnostics.append(item);
  }
}

function ensureCanvasSize() {
  const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
  const width = Math.max(1, Math.floor(canvas.clientWidth * pixelRatio));
  const height = Math.max(1, Math.floor(canvas.clientHeight * pixelRatio));

  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
    if (context && device && format) {
      context.configure({ device, format, alphaMode: "opaque" });
    }
  }

  resolutionMetric.textContent = `${width} × ${height}`;
  return { width, height };
}

async function compileShader() {
  if (!device || !pipelineLayout || !format) {
    return;
  }

  compileButton.disabled = true;
  setStatus("working", "Compiling WGSL…");
  diagnostics.replaceChildren();

  try {
    const module = device.createShaderModule({
      label: `shader-lab ${selectedPreset.id}`,
      code: editor.value,
    });
    const compilationInfo = await module.getCompilationInfo();
    renderDiagnostics(compilationInfo.messages);

    if (compilationInfo.messages.some((message) => message.type === "error")) {
      setStatus("error", "Compilation failed — fix the WGSL and compile again.");
      return;
    }

    const nextPipeline = await device.createRenderPipelineAsync({
      label: "shader-lab live pipeline",
      layout: pipelineLayout,
      vertex: {
        module,
        entryPoint: "vs_main",
      },
      fragment: {
        module,
        entryPoint: "fs_main",
        targets: [{ format }],
      },
      primitive: {
        topology: "triangle-list",
      },
    });

    pipeline = nextPipeline;
    startTime = performance.now();
    setStatus("ready", "Compiled — rendering the current WGSL source.");
  } catch (error) {
    setStatus("error", error instanceof Error ? error.message : "The WebGPU pipeline could not be created.");
  } finally {
    compileButton.disabled = false;
  }
}

function renderFrame(timestamp) {
  if (!device || !context || !uniformBuffer) {
    return;
  }

  const frameStart = performance.now();
  const { width, height } = ensureCanvasSize();

  if (!paused && pipeline) {
    const elapsedSeconds = (timestamp - startTime) / 1000;
    const speed = Number(speedInput.value);
    const uniforms = new Float32Array([
      width,
      height,
      elapsedSeconds,
      speed,
      pointer[0],
      pointer[1],
      0,
      0,
    ]);
    device.queue.writeBuffer(uniformBuffer, 0, uniforms);

    const encoder = device.createCommandEncoder({ label: "shader-lab frame encoder" });
    const pass = encoder.beginRenderPass({
      label: "shader-lab render pass",
      colorAttachments: [
        {
          view: context.getCurrentTexture().createView(),
          clearValue: { r: 0.02, g: 0.025, b: 0.04, a: 1 },
          loadOp: "clear",
          storeOp: "store",
        },
      ],
    });
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.draw(3);
    pass.end();
    device.queue.submit([encoder.finish()]);
  }

  const cpuFrameMs = performance.now() - frameStart;
  frameSamples.push(cpuFrameMs);
  if (frameSamples.length >= 30) {
    const average = frameSamples.reduce((sum, sample) => sum + sample, 0) / frameSamples.length;
    frameMetric.textContent = `${average.toFixed(2)} ms CPU`;
    frameSamples = [];
  }

  animationFrame = requestAnimationFrame(renderFrame);
}

async function initialize() {
  setPreset(selectedPreset);
  setStatus("working", "Requesting WebGPU device…");

  try {
    const gpu = await requestDevice();
    device = gpu.device;
    adapterMetric.textContent = `${gpu.adapter.features.size} features`;

    context = canvas.getContext("webgpu");
    if (!context) {
      throw new Error("This browser exposed WebGPU but could not create a WebGPU canvas context.");
    }

    format = navigator.gpu.getPreferredCanvasFormat();
    uniformBuffer = device.createBuffer({
      label: "shader-lab uniforms",
      size: 32,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    bindGroupLayout = device.createBindGroupLayout({
      label: "shader-lab uniform layout",
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
          buffer: { type: "uniform" },
        },
      ],
    });
    bindGroup = device.createBindGroup({
      label: "shader-lab uniform bind group",
      layout: bindGroupLayout,
      entries: [{ binding: 0, resource: { buffer: uniformBuffer } }],
    });
    pipelineLayout = device.createPipelineLayout({
      label: "shader-lab pipeline layout",
      bindGroupLayouts: [bindGroupLayout],
    });

    ensureCanvasSize();
    await compileShader();

    device.lost.then((info) => {
      setStatus("error", `WebGPU device lost: ${info.message || info.reason}`);
      if (animationFrame) {
        cancelAnimationFrame(animationFrame);
      }
    });

    animationFrame = requestAnimationFrame(renderFrame);
  } catch (error) {
    compileButton.disabled = true;
    pauseButton.disabled = true;
    resetButton.disabled = true;
    speedInput.disabled = true;
    setStatus("error", error instanceof Error ? error.message : "WebGPU initialization failed.");
  }
}

presetSelect.addEventListener("change", () => {
  setPreset(presetById(presetSelect.value));
  void compileShader();
});

compileButton.addEventListener("click", () => {
  void compileShader();
});

pauseButton.addEventListener("click", () => {
  paused = !paused;
  pauseButton.textContent = paused ? "Resume" : "Pause";
  pauseButton.setAttribute("aria-pressed", String(paused));
});

resetButton.addEventListener("click", () => {
  setPreset(selectedPreset);
  void compileShader();
});

canvas.addEventListener("pointermove", (event) => {
  const bounds = canvas.getBoundingClientRect();
  if (bounds.width <= 0 || bounds.height <= 0) {
    return;
  }

  pointer = [
    Math.min(1, Math.max(0, (event.clientX - bounds.left) / bounds.width)),
    Math.min(1, Math.max(0, 1 - (event.clientY - bounds.top) / bounds.height)),
  ];
});

window.addEventListener("beforeunload", () => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame);
  }
  device?.destroy();
});

void initialize();
