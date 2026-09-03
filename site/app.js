import { formatLimit, preferredCanvasFormat, requestAdapter } from "./webgpu.js";

const statusDot = document.querySelector("#status-dot");
const statusTitle = document.querySelector("#status-title");
const statusDetail = document.querySelector("#status-detail");
const metrics = document.querySelector("#metrics");
const featureCount = document.querySelector("#feature-count");
const maxBindGroups = document.querySelector("#max-bind-groups");
const maxTextureSize = document.querySelector("#max-texture-size");
const canvasFormat = document.querySelector("#canvas-format");

if (
  !(statusDot instanceof HTMLElement) ||
  !(statusTitle instanceof HTMLElement) ||
  !(statusDetail instanceof HTMLElement) ||
  !(metrics instanceof HTMLElement) ||
  !(featureCount instanceof HTMLElement) ||
  !(maxBindGroups instanceof HTMLElement) ||
  !(maxTextureSize instanceof HTMLElement) ||
  !(canvasFormat instanceof HTMLElement)
) {
  throw new Error("Shader Lab capability probe markup is incomplete.");
}

async function inspectWebGpu() {
  try {
    const adapter = await requestAdapter();
    statusDot.dataset.state = "ready";
    statusTitle.textContent = "WebGPU adapter available";
    statusDetail.textContent =
      "This browser can run the live Shader Lab experiment. Hardware-dependent results remain canary evidence.";
    featureCount.textContent = formatLimit(adapter.features.size);
    maxBindGroups.textContent = formatLimit(adapter.limits.maxBindGroups);
    maxTextureSize.textContent = `${formatLimit(adapter.limits.maxTextureDimension2D)} px`;
    canvasFormat.textContent = preferredCanvasFormat();
    metrics.hidden = false;
  } catch (error) {
    statusDot.dataset.state = "missing";
    statusTitle.textContent = "Live WebGPU unavailable";
    statusDetail.textContent = error instanceof Error ? error.message : "The browser rejected the WebGPU probe.";
  }
}

void inspectWebGpu();
