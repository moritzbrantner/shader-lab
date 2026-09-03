import { formatLimit, preferredCanvasFormat, requestAdapter } from "./webgpu.js";

const status = document.querySelector("#analysis-status");
const summary = document.querySelector("#analysis-summary");
const limitsBody = document.querySelector("#limits-body");
const features = document.querySelector("#feature-list");
const format = document.querySelector("#preferred-format");

if (
  !(status instanceof HTMLElement) ||
  !(summary instanceof HTMLElement) ||
  !(limitsBody instanceof HTMLElement) ||
  !(features instanceof HTMLElement) ||
  !(format instanceof HTMLElement)
) {
  throw new Error("Shader Lab analysis markup is incomplete.");
}

const limitDefinitions = [
  ["Max bind groups", "maxBindGroups"],
  ["Max bindings / bind group", "maxBindingsPerBindGroup"],
  ["Max 2D texture dimension", "maxTextureDimension2D"],
  ["Max uniform buffer binding", "maxUniformBufferBindingSize"],
  ["Max storage buffer binding", "maxStorageBufferBindingSize"],
  ["Max vertex buffers", "maxVertexBuffers"],
  ["Max vertex attributes", "maxVertexAttributes"],
  ["Max compute workgroup size X", "maxComputeWorkgroupSizeX"],
  ["Max compute invocations / workgroup", "maxComputeInvocationsPerWorkgroup"],
];

async function inspect() {
  try {
    const adapter = await requestAdapter();
    status.dataset.state = "ready";
    status.textContent = "Adapter available";
    summary.textContent =
      "These values come from this browser/driver adapter. They are useful compatibility evidence, not a repository-wide performance baseline.";
    format.textContent = preferredCanvasFormat();

    for (const [label, key] of limitDefinitions) {
      const row = document.createElement("tr");
      const name = document.createElement("th");
      const value = document.createElement("td");
      name.scope = "row";
      name.textContent = label;
      value.textContent = formatLimit(adapter.limits[key]);
      row.append(name, value);
      limitsBody.append(row);
    }

    const sortedFeatures = [...adapter.features].sort((left, right) => left.localeCompare(right));
    if (sortedFeatures.length === 0) {
      const item = document.createElement("li");
      item.textContent = "No optional adapter features reported.";
      features.append(item);
    } else {
      for (const feature of sortedFeatures) {
        const item = document.createElement("li");
        item.textContent = feature;
        features.append(item);
      }
    }
  } catch (error) {
    status.dataset.state = "error";
    status.textContent = "WebGPU unavailable";
    summary.textContent = error instanceof Error ? error.message : "The browser rejected the WebGPU probe.";
    format.textContent = "unavailable";
  }
}

void inspect();
