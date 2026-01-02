# UI Architecture Constraints (GPU Canvas, Hybrid Rendering, Parity)

**Status:** Canonical constraints.

## 1) Goals

- 60 FPS minimum; 120 FPS target on capable hardware
- 10k+ nodes on overview boards without UI collapse
- realtime updates without blocking rendering

## 2) Rendering approach

- GPU-accelerated rendering with wgpu-level control (directly or via a framework that exposes wgpu).
- **Hybrid paradigm:**
  - retained-mode scene graph for canvas nodes/edges
  - immediate-mode widgets for panels, inspectors, logs

## 3) Scale techniques (required)

- virtualization (render only visible nodes)
- LOD rendering when zoomed out (simplify shapes/text)
- throttled/incremental layout
- stable caching of text and geometry

## 4) Desktop vs Web

- Prefer a single Rust UI codebase that can compile to native and WASM.
- Web UI is enterprise-first (remote server).
- Desktop may have extra local-only affordances (FS integration), but the intent is parity.

## 5) Daemon separation

- Default: daemon is a separate process for robustness and server parity.
- Embedded mode is supported for development/testing.

## 6) Realtime data model

- UI reads from projections and subscribes to event streams.
- UI must support catch-up from cursors to handle reconnects.
