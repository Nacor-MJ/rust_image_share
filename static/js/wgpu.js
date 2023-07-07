import init from "./wgpu/pkg/wgpu.js";

init().then(() => {
    console.log("WASM Loaded");
});