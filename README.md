# Asciiquarium (Rust + egui)

A stateless, themeable Asciiquarium widget for `egui`. The widget renders a classic ASCII aquarium using a single `egui::Label`, while your parent application owns and updates the animation state.

- Stateless: The widget only renders from state and assets.
- Themeable: No hardcoded styles. Colors and wrapping are derived from a theme you pass in.
- Simple physics: Fish move with velocity and bounce off the aquarium edges.
- No panics, no unwrap/expect, no unnecessary clones.

Archived originals from the Perl-based Asciiquarium are kept under `archive/original/` for reference.

## Quickstart

1) Add the dependency in your `Cargo.toml`:

    [dependencies]
    egui = "0.27"
    asciiquarium_rust = { git = "https://github.com/gitcoder89431/asciiquarium-rust", tag = "v0.1.0" }

2) Prepare assets and state in your app:

    use asciiquarium_rust::{
        get_fish_assets, update_aquarium, AsciiquariumTheme, AsciiquariumWidget,
        AquariumState, FishInstance,
    };

    // Build assets once (e.g., at startup).
    let assets = get_fish_assets();

    // Create your aquarium state (owned by the parent app).
    let mut state = AquariumState {
        size: (80, 24), // character grid width x height
        fishes: vec![
            FishInstance {
                fish_art_index: 0,
                position: (2.0, 3.0),
                velocity: (0.4, 0.0),
            },
            FishInstance {
                fish_art_index: 1,
                position: (30.0, 10.0),
                velocity: (-0.3, 0.1),
            },
        ],
    };

3) In your app’s update loop, update and render:

    // Update the simulation (e.g., once per frame or on your own tick).
    update_aquarium(&mut state, &assets);

    // Derive styles from your theme (no hardcoded styles).
    let theme = AsciiquariumTheme {
        text_color: egui::Color32::from_rgb(180, 220, 255),
        background: Some(egui::Color32::from_rgb(8, 12, 16)),
        wrap: false, // keep ASCII grid alignment
    };

    // Render: a single monospace label with your aquarium.
    ui.add(AsciiquariumWidget {
        state: &state,
        assets: &assets,
        theme: &theme,
    });

## Theming

Follow the “Design for Theming” rule: the component does not hardcode colors or styles. Everything flows through `AsciiquariumTheme`.

- `text_color`: The color used for the ASCII characters.
- `background`: Optional background fill for the label area.
- `wrap`: Line wrapping for the label (usually `false` to preserve ASCII alignment).

Example themes:

    // Light theme
    let light = AsciiquariumTheme {
        text_color: egui::Color32::from_rgb(40, 40, 40),
        background: Some(egui::Color32::from_rgb(245, 245, 245)),
        wrap: false,
    };

    // High contrast
    let high_contrast = AsciiquariumTheme {
        text_color: egui::Color32::WHITE,
        background: Some(egui::Color32::BLACK),
        wrap: false,
    };

## API Overview

- `FishArt`:
  - `art: &'static str`
  - `width: usize`
  - `height: usize`

- `FishInstance`:
  - `fish_art_index: usize`
  - `position: (f32, f32)`  // top-left in character coordinates
  - `velocity: (f32, f32)`  // characters per tick

- `AquariumState`:
  - `size: (usize, usize)`  // width x height in characters
  - `fishes: Vec<FishInstance>`

- Functions:
  - `get_fish_assets() -> Vec<FishArt>`
  - `update_aquarium(state: &mut AquariumState, assets: &[FishArt])`
  - `render_aquarium_to_string(state: &AquariumState, assets: &[FishArt]) -> String`

- Widget:
  - `AsciiquariumWidget<'a> { state: &'a AquariumState, assets: &'a [FishArt], theme: &'a AsciiquariumTheme }`
  - Implements `egui::Widget` and renders a single, monospace label.

## Design Notes

- Stateless rendering: The widget takes immutable `&AquariumState` and `&[FishArt]` and renders a single string. No side effects, no mutation.
- Parent-managed animation: The parent application updates `AquariumState` each tick using `update_aquarium`.
- Float-to-int: Rendering uses `floor()` for stable projection and less jitter.
- Bounds and clipping: Rendering clips safely; later fish in the slice overdraw earlier ones.
- Dimensions: `AquariumState.size` is in character cells. Choose a fixed grid (e.g., 80x24) or set it based on your layout needs.

## Testing

Run unit tests:

    cargo test

Tests cover:
- Edge bounce behavior
- Left-edge clipping in rendering
- Asset measurement correctness

## Tips & Troubleshooting

- Misaligned ASCII: Ensure `theme.wrap` is `false` and that the container does not force wrapping. The widget uses `RichText::monospace()`.
- Too small or clipped label: The rendered string’s dimensions are exactly `size.1` lines by `size.0` columns. Place it in a container large enough to display without wrapping or scaling.
- Frame timing: If motion is too fast or slow, adjust fish velocities or call `update_aquarium` at your preferred tick rate.

## Roadmap

- Additional fish and sea creatures from the classic Asciiquarium
- Configurable z-ordering and layering
- Optional wrap-around movement
- Simple scene randomizer utilities (spawn fish with random velocity and positions)

## Contributing

- Follow the Rust rules in `rust_rules.md`:
  - No `unwrap`/`expect` in application logic
  - No panics; handle errors gracefully
  - No unnecessary clones; prefer references
  - Keep modules small and focused; single responsibility
  - Rendering is a pure function of state
  - Design for theming (no hardcoded styles)
- Use `rustfmt` and `clippy` with zero warnings.

## License and Credits

- The original Perl Asciiquarium (Kirk Baucom) materials are archived under `archive/original/`.
- This crate provides an `egui`-based Rust implementation with a stateless, themeable widget design.
- ASCII art in this crate is a minimal starter set for demonstration. Expand or replace as needed per your project’s licensing requirements.

## Features

- Stateless egui widget: renders to a single monospace label string
- Deterministic animation loop with fixed timestep for smooth pacing
- Environment:
  - Waterlines with subtle wave motion
  - Seaweed with gentle sway
  - Castle at bottom-right
  - Ship at the surface; shark and whale underwater with spout animation
  - Fish bubbles (desynced per fish)
- Fish behavior:
  - Bounce physics with occasional direction variance on wall bounces
  - Schools: groups traverse and despawn off-screen
  - Orientation correction: fish ASCII auto-mirrors so they face their travel direction
- Despawn and respawn cycles for large entities (ship, shark, whale)
- Minimal defaults, no required configuration
- Tests and CI (rustfmt, clippy, build, test)

## Colorized rendering (optional)

By default, the widget renders a plain, single-color ASCII string. You can opt into a colorized renderer that maps certain glyphs to colors using a palette:

- Enable by setting `enable_color = true` and providing a `palette`
- Renders via an internal color `LayoutJob` while keeping the API unchanged
- Glyph color mapping (initial pass):
  - Water surface: `~` and `^` → `palette.water`
  - Seaweed: `(` and `)` → `palette.seaweed`
  - Bubbles: `.` → `palette.bubble`
  - Original mask placeholders: `?` → `palette.water_trail` (to mimic motion trails)
  - All other glyphs default to `theme.text_color` (castle, ship, fish body, etc.)

Example (palette + colorized theme):

    use asciiquarium_rust::{AsciiquariumTheme};
    use asciiquarium_rust::widgets::asciiquarium::AsciiquariumPalette;

    let palette = AsciiquariumPalette {
        water: egui::Color32::from_rgb(120, 180, 255),
        water_trail: egui::Color32::from_rgba_unmultiplied(120, 180, 255, 120),
        seaweed: egui::Color32::from_rgb(60, 180, 120),
        castle: egui::Color32::from_rgb(200, 200, 200),
        ship: egui::Color32::from_rgb(230, 230, 230),
        bubble: egui::Color32::from_rgb(200, 230, 255),
        shark: egui::Color32::from_rgb(180, 200, 210),
        whale: egui::Color32::from_rgb(160, 190, 210),
        fish: egui::Color32::from_rgb(255, 200, 120),
    };

    let theme = AsciiquariumTheme {
        text_color: egui::Color32::from_rgb(180, 220, 255),
        background: Some(egui::Color32::from_rgb(8, 12, 16)),
        wrap: false,
        enable_color: true,
        palette: Some(palette),
    };

Then render as usual with `AsciiquariumWidget { state, assets, theme }`.

Notes:
- Color mapping will evolve. Future iterations may introduce per-art or per-part mapping for higher fidelity while keeping defaults minimal.
- When `enable_color = false` or `palette = None`, the widget falls back to plain text rendering.

## Notes on original mask characters

The classic Asciiquarium art uses special characters (like `?`) in separate color mask layers. In this crate:

- With color disabled (default), mask characters are skipped in rendering for a clean monochrome output.
- With color enabled, mask characters render using `palette.water_trail` to mimic subtle motion trails behind large entities.
