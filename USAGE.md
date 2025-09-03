# USAGE

A concise guide to integrating the Asciiquarium widget into your `egui` app.

This widget is stateless: your parent application owns and updates the animation state. Styling is derived from a passed-in theme (no hardcoded styles).

## 1) Add dependency

In your application's `Cargo.toml`:

```toml
[dependencies]
egui = "0.27"
asciiquarium_rust = { path = "." } # Or your git/registry source
```

## 2) Import and initialize

Create (or reuse) your assets once at startup.

```rust
use asciiquarium_rust::{
    get_fish_assets, update_aquarium, AsciiquariumTheme, AsciiquariumWidget,
    AquariumState, FishInstance,
};

pub struct MyApp {
    assets: Vec<asciiquarium_rust::FishArt>,
    state: AquariumState,
    theme: AsciiquariumTheme,
}

impl MyApp {
    pub fn new() -> Self {
        let assets = get_fish_assets();

        // Choose an initial grid size. You can make this dynamic later.
        let size = (80, 24);

        let state = AquariumState {
            size,
            fishes: vec![
                FishInstance { fish_art_index: 0, position: (2.0, 3.0), velocity: (0.35, 0.0) },
                FishInstance { fish_art_index: 1, position: (30.0, 10.0), velocity: (-0.25, 0.08) },
            ],
        };

        let theme = AsciiquariumTheme {
            text_color: egui::Color32::from_rgb(180, 220, 255),
            background: Some(egui::Color32::from_rgb(8, 12, 16)),
            wrap: false, // keep ASCII grid aligned
        };

        Self { assets, state, theme }
    }
}
```

## 3) Update and render (egui/eframe)

Call `update_aquarium` each frame or at your preferred tick rate, then render the widget.

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, ui_frame: &mut eframe::Frame) {
        // Drive animation: update state using the assets' dimensions.
        update_aquarium(&mut self.state, &self.assets);

        // Optionally control repaint rate for smoother animation (eframe helper).
        ctx.request_repaint_after(std::time::Duration::from_millis(33));

        egui::CentralPanel::default().show(ctx, |ui| {
            // Render as a single monospace label.
            ui.add(AsciiquariumWidget {
                state: &self.state,
                assets: &self.assets,
                theme: &self.theme,
            });
        });
    }
}
```

## 4) Theming (no hardcoded styles)

Derive all styles from a theme you pass to the widget.

```rust
let dark_theme = AsciiquariumTheme {
    text_color: egui::Color32::from_rgb(180, 220, 255),
    background: Some(egui::Color32::from_rgb(8, 12, 16)),
    wrap: false,
};

let light_theme = AsciiquariumTheme {
    text_color: egui::Color32::from_rgb(40, 40, 40),
    background: Some(egui::Color32::from_rgb(245, 245, 245)),
    wrap: false,
};
```

Swap the theme at runtime to adapt to different palettes or modes (light/dark/high-contrast).

## 5) Grid sizing

The widget renders a fixed grid: `AquariumState.size = (width_chars, height_chars)`.

- Keep `wrap = false` to preserve alignment.
- Ensure the container can display the full grid (e.g., size (80, 24) prints 24 lines of 80 characters).
- You can dynamically compute `size` from available pixels and the current monospace font metrics, or keep it fixed.

To resize the grid dynamically:

```rust
fn set_grid_size(&mut self, width_chars: usize, height_chars: usize) {
    self.state.size = (width_chars, height_chars);
}
```

## 6) Randomized fish helper (optional utility)

A quick helper to add a fish at a given index with a random velocity and position.

```rust
fn spawn_fish(state: &mut AquariumState, fish_art_index: usize) {
    let (w, h) = state.size;
    let x = (rand::random::<f32>() * (w as f32 - 1.0)).max(0.0);
    let y = (rand::random::<f32>() * (h as f32 - 1.0)).max(0.0);
    let vx = (rand::random::<f32>() - 0.5) * 0.8; // ~[-0.4, 0.4]
    let vy = (rand::random::<f32>() - 0.5) * 0.4; // ~[-0.2, 0.2]
    state.fishes.push(FishInstance {
        fish_art_index,
        position: (x, y),
        velocity: (vx, vy),
    });
}
```

Note: Add `rand = "0.8"` to your app’s `Cargo.toml` if you use this.

## Tips

- Overdraw order: later fish in `state.fishes` are drawn atop earlier ones.
- Motion stability: rendering uses `floor()` for float-to-int projection to reduce jitter.
- Clipping: drawing is clipped at grid boundaries; partial fish are handled gracefully.
- Performance: all logic is simple; keep heavy work out of your render/update loop.
- Lint/format: run `cargo fmt` and `cargo clippy` (aim for zero warnings).

## Minimal end-to-end snippet

```rust
use asciiquarium_rust::*;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Asciiquarium Demo",
        native_options,
        Box::new(|_cc| Box::new(MyApp::new())),
    )
}

struct MyApp {
    assets: Vec<FishArt>,
    state: AquariumState,
    theme: AsciiquariumTheme,
}

impl MyApp {
    fn new() -> Self {
        let assets = get_fish_assets();
        let state = AquariumState {
            size: (80, 24),
            fishes: vec![
                FishInstance { fish_art_index: 0, position: (2.0, 3.0), velocity: (0.35, 0.0) },
                FishInstance { fish_art_index: 1, position: (30.0, 10.0), velocity: (-0.25, 0.08) },
            ],
        };
        let theme = AsciiquariumTheme {
            text_color: egui::Color32::from_rgb(180, 220, 255),
            background: Some(egui::Color32::from_rgb(8, 12, 16)),
            wrap: false,
        };
        Self { assets, state, theme }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        update_aquarium(&mut self.state, &self.assets);
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(AsciiquariumWidget { state: &self.state, assets: &self.assets, theme: &self.theme });
        });
    }
}
```
