export class WebGpuUnavailableError extends Error {
  constructor(message) {
    super(message);
    this.name = "WebGpuUnavailableError";
  }
}

export async function requestAdapter() {
  if (!("gpu" in navigator)) {
    throw new WebGpuUnavailableError(
      "WebGPU is not exposed by this browser. Use a current browser with WebGPU enabled to run the live experiments.",
    );
  }

  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) {
    throw new WebGpuUnavailableError(
      "WebGPU is exposed, but the browser could not provide a compatible GPU adapter.",
    );
  }

  return adapter;
}

export async function requestDevice() {
  const adapter = await requestAdapter();
  const device = await adapter.requestDevice();
  return { adapter, device };
}

export function preferredCanvasFormat() {
  if (!("gpu" in navigator)) {
    return "unavailable";
  }

  return navigator.gpu.getPreferredCanvasFormat();
}

export function formatLimit(value) {
  if (typeof value === "bigint") {
    return value.toLocaleString("en-US");
  }

  if (typeof value === "number") {
    return value.toLocaleString("en-US");
  }

  return String(value ?? "—");
}
