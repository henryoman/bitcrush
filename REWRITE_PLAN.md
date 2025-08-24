# Bitcrush desktop rewrite plan (Rust engine + Tauri 2, Vite + TS UI shell)

This document outlines the minimum plan to rebuild Bitcrush as a desktop‑only app using:

- Bun to run Vite scripts
- TypeScript UI shell (no styling scope)
- Tauri 2 desktop shell with IPC
- Rust-native image engine for all processing and exports

The plan prioritizes feature parity and performance while keeping the architecture simple and maintainable. No extra features are added beyond achieving parity and a clean desktop experience.

---

## 1) Scope

- Goals
  - Desktop‑only parity: image input, algorithm selection, grid size, palette, preview, export.
  - UI is minimal; Rust performs all image work.
  - Tauri 2 for file dialogs and filesystem.
- Non‑goals
  - No TS/WebWorker engine.
  - No GPU in v1.
  - No UI styling.
- Acceptance criteria
  - Side‑by‑side comparison with the current app produces visually equivalent outputs across all algorithms on a set of reference images.
  - Desktop builds run on macOS (primary), with configuration ready for Windows/Linux builds.
  - PNG export works for both base resolution and upscaled images.

---

## 2) What we’re porting

- Algorithms: port all to Rust with the same names/behavior.
- Pipeline: scale to grid, process, upscale/center to 640 px display — in Rust.
- Config: palettes in `config/palettes.ts` and additional palette files in `config/gpl-palettes/`.
- UI: Next.js components for algorithm selection, grid size, palette picker, uploader, preview, downloads.
- Hook: `usePixelization.ts` orchestrates processing and dual output (base and upscaled).

Parity scope is defined by the above. Visual equivalence is measured on base grid and the upscaled 640‑px display rendering.

---

## 3) Architecture (minimal)

- UI shell (TS): minimal DOM bindings; IPC to Rust.
- Tauri 2 + Rust engine: all algorithms + I/O.
- Commands: `load_image`, `set_params`, `render_preview`, `render_base`, `export_png`, `list_palettes`.

### Data flow (minimal)

- UI selects image → Rust decodes/processes → returns preview PNG bytes → UI displays → export writes via Rust.

---

## 4) Tech stack

- Bun (run Vite and scripts), Vite 6, TS 5 for UI shell
- Tauri 2 + Rust 1.7x
- Rust crates: `image`, `rayon`, optional SIMD (`wide`), `tauri`, `serde`, `toml`


---

## 5) Project layout

```text
bitcrush/
  index.html                  # Root HTML (Vite serves this)
  src/
    assets/
      (icons, logos, etc.)
    styles.css                # Global styles
    main.ts                   # UI bootstrap; binds DOM and IPC
    state/
      appState.ts             # UI state (selected algo, palette, grid)
      types.ts                # Minimal IPC types
    ui/
      controls.ts             # Minimal: wire inputs to IPC
      preview.ts              # Render PNG bytes from Rust
    # No TS palettes config; palettes come from Rust via IPC

  dist/                       # Vite build output (frontendDist)

  src-tauri/
    capabilities/
      default.json            # Capabilities for main window
    resources/
      palettes/               # All palettes live here
        gpl/                  # Drop .gpl files here
        palettes.toml         # Additional palettes defined in TOML
    src/
      lib.rs                  # Tauri builder and commands
      main.rs                 # Entry, calls run()
      engine/
        lib.rs                # Rust image engine (all algorithms)
        algorithms/           # Ports of algorithms
        color.rs
        quant.rs
        dither/
          floyd_steinberg.rs
          ordered.rs
          selective.rs
          edge.rs
          dual_color.rs
        pipeline.rs
        io.rs
    Cargo.toml
    tauri.conf.json           # Tauri 2 configuration

  package.json                # Vite scripts executed by Bun
  vite.config.ts              # Dev server: port 1420, HMR 1421; ignore src-tauri
  tsconfig.json
  README.md                   # Dev/build instructions
```

---
\\
---

## 7) IPC surfaces (what UI calls)

- `load_image(path | bytes)`
- `set_params({ algorithmName, gridSize, paletteId })`
- `render_preview()` → PNG bytes
- `render_base()` → PNG bytes
- `export_png(path, { variant: "preview" | "base" })`
- `list_palettes()` → core + .gpl palettes (names + colors)

---

## 8) Pixelization engine (Rust)

- Pipeline: decode → resize to grid → algorithm process → upscale to 640 → encode PNG
- Registry mirrors existing names/descriptions

### Performance notes

- `rayon` parallel rows/tiles; optional SIMD (`wide`)
- Reuse buffers; avoid reallocations; stride-aware loops
- Previews as PNG bytes; exports written in Rust

---

## 9) (Optional later) GPU path

- If needed, evaluate `wgpu` for upscale or selected kernels; not in v1

---

## 10) File handling and palettes

- Open image
  - Primary: standard `<input type="file">` / drag‑drop with object URLs.
  - Optional (desktop‑native): Tauri `dialog::FileDialog` to select a file and read via `fs` if needed.
- Save image
  - Use Tauri `dialog` (save) to choose a path and `fs` to write PNG bytes.
- Palettes
  - `.gpl` files under `src-tauri/resources/palettes/gpl/` loaded at runtime
  - `palettes.toml` under `src-tauri/resources/palettes/` for additional palettes
  - `list_palettes` merges all sources and returns names + colors

---

## 11) Types and parameters

- Mirror existing types; define once in Rust and a minimal TS mirror for UI only:
  - `PixelizationParams` (gridSize, palette name or id, algorithm name)
  - `ColorPalette` (name, hex colors)
  - `Algorithm` (name, description)
- Source of truth is Rust. TS types are generated or hand‑kept minimal for IPC payloads.

---

## 12) Testing (minimal)

- Rust: a few small golden fixtures per algorithm to guard regressions
- Manual UI smoke: load → preview → base → export

---

## 13) Migration

- `lib/algorithms/**/*` → `src-tauri/src/engine/algorithms/**/*` (Rust)
- `lib/pixelizer/pixelizeImage.ts` → `src-tauri/src/engine/pipeline.rs`
- `config/palettes.ts` → `src-tauri/src/engine/palettes.rs` (embedded)

---

## 14) Step-by-step implementation plan

1. Repo bootstrap
   - Initialize Tauri 2 app (`src-tauri/`), Bun project, TS config, and basic `index.html`.
   - Add Bun scripts: `dev`, `build`, `test`.
2. Minimal UI shell
   - Static HTML layout, CSS tokens, and controls placeholders.
3. Rust engine scaffolding
   - Create `src-tauri/src/engine` with pipeline, algorithms, color/quant modules.
   - Add Tauri commands for image load/render/export.
4. Port algorithms to Rust
   - Implement Standard, Enhanced, Artistic, Bayer, Floyd-Steinberg, Dual Color, Edge, Selective, Ordered Selective, Randomized Selective.
5. Palettes
   - Embed core palettes; implement `.gpl` loader; expose to UI.
6. UI wiring: call IPC; render preview/base PNG bytes
7. Packaging: `bun run build` + `cargo tauri build`

---

## 15) Timeline (indicative)

- Week 1: Bootstrap, engine scaffolding, IPC commands
- Week 2: Port algorithms, embed palettes
- Week 3: Previews/exports + packaging

---

## 16) Risks and mitigations

- IPC throughput and memory copies
  - Keep previews as PNG bytes; write large exports directly from Rust.
- Pixel parity differences (color rounding/sampling)
  - Use small fixtures and edge‑case tests; keep math identical to original.
- Tauri permission friction
  - Enable only required APIs; keep file operations explicit.

---

## 17) Developer commands (reference)

```bash
# Install deps
bun install

# Run desktop app in dev (spawns Bun dev server via Tauri beforeDevCommand)
cargo tauri dev

# Build frontend assets
bun run build

# Package desktop app
cargo tauri build

# Run tests
bun test
```

This plan is deliberately incremental to reach feature parity quickly, keep risk low, and leave room for an optional native engine once parity and packaging are solid.
