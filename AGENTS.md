# Tileline HPC Compute & Graphics Engine (GMS) - Agent Coordination Plan

## Overview
This file coordinates the architectural migration of the GMS graphics library from `wgpu` to pure Vulkan, Metal, HIP/ROCm, and CUDA. The main goal is to remove high-level graphics APIs while keeping the public API exactly the same.

## Core Rules & Constraints
1. **API Compatibility:** GMS's external calls must remain exactly the same to ensure backward compatibility.
2. **Modern Dependencies:** Adopt the latest `sxrc` and `mps` libraries. Do NOT use Rayon or Bevy.
3. **CUDA Backend:** Use `cudarc` (by chelsea0x3b) for the CUDA implementation.
4. **Vulkan Backend:** Use Raw Vulkan.
5. **No High-Level Graphics APIs:** `wgpu` and similar high-level APIs are strictly forbidden.
6. **Metal Backend:** Use Raw Metal for the Mac environment.
7. **Memory Optimization:** Use `SXRC` for memory savings.
8. **Strict Adherence:** Do not deviate from the user's instructions.
9. **English Language:** All comments, documentation, and variable names in the codebase must be written in English.
10. **Documentation Requirement:** All new architectural features, non-trivial functions, and major structures must be thoroughly documented using Rust docstrings (`///`). Code without adequate documentation will be rejected.

## Agent Roles
(To be defined as the plan progresses...)
