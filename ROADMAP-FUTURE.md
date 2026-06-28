# GMS (Tileline HPC Compute & Graphics Engine) - Future Roadmap

## Overview
This document outlines the long-term architectural vision and milestones for the GMS engine. As we migrate to a raw hardware abstraction layer (HAL) bypassing high-level wrappers like `wgpu`, our focus shifts to providing both seamless portability and uncompromising performance.

## 1. Unified Compute & Shader Languages
In the future, GMS will feature a **fully unified compute and shader pipeline**.
- **Shared Codebase:** Vulkan Compute, CUDA, and Metal will eventually share completely common languages (e.g., a unified front-end like WGSL/HLSL that compiles down to SPIR-V, PTX, and MSL under the hood).
- **Write Once, Run Everywhere:** The default developer experience will be abstracted, ensuring that standard rendering and compute tasks run optimally across all supported hardware without rewriting shaders.

## 2. Optional Direct API Calls (Native Escapes)
While unification guarantees portability, GMS will never limit developers who need hardware-specific optimizations.
- **Raw Access:** The architecture will allow developers to bypass the common language layer and use **optional direct calls**.
- **Specialized Kernels:** If a developer writes a custom, highly-optimized CUDA kernel (`.cu` -> `.ptx`) or a specialized Metal shader, GMS will provide native escape hatches to plug these directly into the dispatch pipeline.
- This hybrid model ensures GMS remains a "Swiss Army knife"—approachable for general use cases, but limitlessly powerful for specialized HPC workloads.

## 3. Asynchronous Memory Streaming (Megatextures)
- Fully functional implementation of the **VRAM Paged Memory System** (Software Virtual Memory).
- Seamless integration with the SXRC compression library for streaming 4GB+ AI prediction tables and assets (like Order-4 Contexts) without triggering Out-Of-Memory errors on integrated GPUs.

## Future Milestones
- [ ] Build the unified compiler bridge to map common shaders to PTX (CUDA), MSL (Metal), and SPIR-V (Vulkan).
- [ ] Finalize the `virtual_memory` page-fault/streaming implementation across all active backends.
- [ ] Expose the `unsafe` backend bridges in `src/hal/` for developers to inject direct native API calls.
