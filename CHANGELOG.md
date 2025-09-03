# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] – Initial release

Highlights:
- A stateless, themeable Asciiquarium widget for `egui` that renders to a single monospace label.
- Deterministic animation loop with a fixed timestep for smooth, consistent pacing.
- Minimal defaults, no configuration required for a pleasant "classic" experience.

Added
- Core widget and state
  - `AsciiquariumWidget<'a>` renders from `AquariumState` + fish assets.
  - Pure rendering function that composes a fixed-size ASCII grid into a single string.
  - Fixed timestep update with wall-bounce physics and safe clipping (no panics).
- Environment and effects
  - Waterlines with subtle column-varying wave motion.
  - Seaweed stalks with gentle sway and deterministic, seeded placement.
  - Castle at bottom-right.
  - Bubbles emitted by fish (desynced per fish).
- Entities
  - Fish: classic pacing (chars-per-second), correct facing via ASCII mirroring, subtle horizontal jitter, occasional bounce variance, and fish “schools” that traverse and despawn off-screen.
  - Ship (surface), Shark (underwater), Whale (underwater) with spout animation; all despawn by swimming off the viewport, with deterministic respawn timing.
- Theming and color (optional)
  - `AsciiquariumTheme` with monospace text color, background fill, wrapping control.
  - Optional color path (off by default) using an internal `LayoutJob`, enabled via `theme.enable_color = true` with `AsciiquariumPalette`.
  - Color mapping v1:
    - Water (~,^): palette.water
    - Seaweed (()): palette.seaweed
    - Bubbles (.): palette.bubble
    - Mask placeholders from original art (?): palette.water_trail (subtle trails)
    - Other glyphs default to `theme.text_color` (e.g., castle, ship, fish body)
- Assets
  - Manual starter fish set plus an automated extraction of classic fish from the original Perl script (`src/widgets/generated_fish_assets.rs`).
  - Extraction tool: `src/bin/extract_fish.rs` parses the archived original and generates Rust constants.
- Example
  - `examples/egui_demo`: runnable eframe demo with:
    - Grid controls, frame cadence, theme color pickers.
    - Toggle for colorized rendering and live palette editing.
- Docs and repo hygiene
  - `README.md` (Quickstart, Features, Colorized rendering usage), `USAGE.md`.
  - Archived original Perl assets under `archive/original/`.
  - `.gitignore` for build artifacts and internal docs (plan, rules).

Fixed
- Mask artifacts from the original (`?` characters):
  - Monochrome path: `?` is skipped (transparent).
  - Color path: `?` is rendered with `palette.water_trail` to mimic gentle motion trails.
- CI formatting failures (`cargo fmt`) and lints (`clippy -D warnings`) addressed.

CI
- GitHub Actions workflow:
  - rustfmt check
  - clippy (deny warnings)
  - build (Linux/macOS/Windows)
  - tests

Compatibility
- Rust edition: 2021
- `egui = "0.27"`

Notes
- The color mapping is intentionally simple in v1 and will evolve. A future pass may support finer-grained per-art coloring while keeping the current public API stable.
- All animation remains deterministic and side-effect free inside the widget; the parent application owns state and calls update + render.
