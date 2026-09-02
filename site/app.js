const statusDot = document.querySelector("#status-dot");
const statusTitle = document.querySelector("#status-title");
const statusDetail = document.querySelector("#status-detail");
const metrics = document.querySelector("#metrics");
const featureCount = document.querySelector("#feature-count");
const maxBindGroups = document.querySelector("#max-bind-groups");
const maxTextureSize = document.querySelector("#max-texture-size");

if (
  !(statusDot instanceof HTMLElement) ||
  !(statusTitle instanceof HTMLElement) ||
  !(statusDetail instanceof HTMLElement) ||
  !(metrics instanceof HTMLElement) ||
  !(featureCount instanceof HTMLElement) ||
  !(maxBindGroups instanceof HTMLElement) ||
  !(maxTextureSize instanceof HTMLElement)
) {
  throw new Error("Shader Lab capability probe markup is incomplete.");
}

async function inspectWebGpu() {
  if (!("gpu" in navigator)) {
    statusDot.dataset.state = "missing";
    statusTitle.textContent = "WebGPU is not exposed by this browser";
    statusDetail.textContent =
      "The Pages explanation still works here. Future live GPU experiments will require a browser with WebGPU enabled.";
    return;
  }

  try {
    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      statusDot.dataset.state = "missing";
      statusTitle.textContent = "WebGPU exists, but no adapter is available";
      statusDetail.textContent =
        "This can happen when the browser, operating system, driver, or execution environment cannot expose a compatible GPU.";
      return;
    }

    statusDot.dataset.state = "ready";
    statusTitle.textContent = "WebGPU adapter available";
    statusDetail.textContent =
      "This browser can host the future WebGPU experiments. No rendering device was requested by this capability probe.";
    featureCount.textContent = String(adapter.features.size);
    maxBindGroups.textContent = String(adapter.limits.maxBindGroups);
    maxTextureSize.textContent = `${adapter.limits.maxTextureDimension2D}px`;
    metrics.hidden = false;
  } catch (error) {
    statusDot.dataset.state = "missing";
    statusTitle.textContent = "WebGPU probe failed";
    statusDetail.textContent = error instanceof Error ? error.message : "The browser rejected the WebGPU probe.";
  }
}

void inspectWebGpu();
