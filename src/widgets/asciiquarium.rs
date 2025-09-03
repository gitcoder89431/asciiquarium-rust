/*!
Asciiquarium widget scaffold for egui.

Agent Log:
- Created core data structures: FishArt, FishInstance, AquariumState.
- Implemented update_aquarium with wall-bounce using asset dimensions.
- Implemented render_aquarium_to_string using a 2D char grid, floor() projection, clipping, and last-wins overlap.
- Added theming via AsciiquariumTheme (no hardcoded colors/styles).
- Added AsciiquariumWidget<'a> implementing egui::Widget; stateless, consumes state + assets + theme.
- Decisions:
  - AquariumState.size is (usize, usize) to simplify indexing; parent code can cast from other units if desired.
  - Float-to-int via floor() for stable, predictable rendering.
  - No unwrap/expect, no panics; graceful handling of bad fish_art_index.
*/

use egui;

/// Visual asset for a fish (ASCII art and its measured dimensions).
#[derive(Debug, Clone, Copy)]
pub struct FishArt {
    pub art: &'static str,
    pub width: usize,
    pub height: usize,
}

/// A single moving fish instance in the aquarium.
#[derive(Debug, Clone)]
pub struct FishInstance {
    /// Index into the assets slice.
    pub fish_art_index: usize,
    /// Top-left position in character coordinates (float for smooth movement).
    pub position: (f32, f32),
    /// Velocity in characters per tick.
    pub velocity: (f32, f32),
}

/// The aquarium state that the parent application owns and updates.
#[derive(Debug, Default)]
pub struct AquariumState {
    /// Bounds of the aquarium in character cells (width, height).
    pub size: (usize, usize),
    /// All fish currently in the aquarium.
    pub fishes: Vec<FishInstance>,
}

/// Theme passed during render. No hardcoded styles in the component.
#[derive(Clone, Debug)]
pub struct AsciiquariumTheme {
    pub text_color: egui::Color32,
    /// Optional background fill for the label area.
    pub background: Option<egui::Color32>,
    /// Whether to wrap lines in the ASCII label. Usually false for grids.
    pub wrap: bool,
}

impl Default for AsciiquariumTheme {
    fn default() -> Self {
        Self {
            text_color: egui::Color32::LIGHT_GRAY,
            background: None,
            wrap: false,
        }
    }
}

/// Update the aquarium by one tick with simple wall-bounce physics.
///
/// Notes:
/// - Uses each fish's asset width/height to bounce at the visible edge.
/// - Keeps fish entirely within bounds after a bounce.
/// - Handles invalid asset indices gracefully by treating size as 1x1.
pub fn update_aquarium(state: &mut AquariumState, assets: &[FishArt]) {
    let (aw, ah) = (state.size.0 as f32, state.size.1 as f32);

    for fish in &mut state.fishes {
        // Integrate position.
        fish.position.0 += fish.velocity.0;
        fish.position.1 += fish.velocity.1;

        // Resolve asset size (fallback to 1x1 if out of range).
        let (fw, fh) = assets
            .get(fish.fish_art_index)
            .map(|a| (a.width as f32, a.height as f32))
            .unwrap_or((1.0, 1.0));

        // Bounce on X.
        if fish.position.0 < 0.0 {
            fish.position.0 = 0.0;
            fish.velocity.0 = fish.velocity.0.abs();
        } else if fish.position.0 + fw > aw {
            fish.position.0 = (aw - fw).max(0.0);
            fish.velocity.0 = -fish.velocity.0.abs();
        }

        // Bounce on Y.
        if fish.position.1 < 0.0 {
            fish.position.1 = 0.0;
            fish.velocity.1 = fish.velocity.1.abs();
        } else if fish.position.1 + fh > ah {
            fish.position.1 = (ah - fh).max(0.0);
            fish.velocity.1 = -fish.velocity.1.abs();
        }
    }
}

/// Render the aquarium state into a single string (newline-separated).
///
/// - Uses floor() for stable float->int projection.
/// - Clips art at boundaries.
/// - Later fish in the list overdraw earlier ones (simple z-order).
pub fn render_aquarium_to_string(state: &AquariumState, assets: &[FishArt]) -> String {
    let (w, h) = state.size;
    if w == 0 || h == 0 {
        return String::new();
    }

    let mut grid = vec![' '; w * h];

    for fish in &state.fishes {
        let art = match assets.get(fish.fish_art_index) {
            Some(a) => a,
            None => continue, // Graceful skip if bad index
        };

        let x0 = fish.position.0.floor() as isize;
        let y0 = fish.position.1.floor() as isize;

        for (dy, line) in art.art.lines().enumerate() {
            let y = y0 + dy as isize;
            if y < 0 || y >= h as isize {
                continue;
            }

            for (dx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = x0 + dx as isize;
                if x < 0 || x >= w as isize {
                    continue;
                }
                let idx = y as usize * w + x as usize;
                grid[idx] = ch;
            }
        }
    }

    // Join into a single string with newline separators.
    let mut out = String::with_capacity((w + 1) * h);
    for row in 0..h {
        let start = row * w;
        let end = start + w;
        out.extend(grid[start..end].iter().copied());
        if row + 1 < h {
            out.push('\n');
        }
    }
    out
}

/// egui widget: stateless, renders from AquariumState + assets + theme.
pub struct AsciiquariumWidget<'a> {
    pub state: &'a AquariumState,
    pub assets: &'a [FishArt],
    pub theme: &'a AsciiquariumTheme,
}

impl<'a> egui::Widget for AsciiquariumWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rendered = render_aquarium_to_string(self.state, self.assets);
        let text = egui::RichText::new(rendered)
            .monospace()
            .color(self.theme.text_color);
        let label = egui::Label::new(text).wrap(self.theme.wrap);

        if let Some(fill) = self.theme.background {
            egui::Frame::default()
                .fill(fill)
                .show(ui, |ui| ui.add(label))
                .response
        } else {
            ui.add(label)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_assets() -> Vec<FishArt> {
        vec![FishArt {
            art: "<>",
            width: 2,
            height: 1,
        }]
    }

    #[test]
    fn bounce_at_right_edge() {
        let assets = mk_assets();
        let mut state = AquariumState {
            size: (10, 3),
            fishes: vec![FishInstance {
                fish_art_index: 0,
                position: (8.5, 1.0),
                velocity: (1.0, 0.0),
            }],
        };
        update_aquarium(&mut state, &assets);
        let f = &state.fishes[0];
        assert!(
            (f.position.0 - 8.0).abs() < 1e-6,
            "x pos should clamp to 8.0"
        );
        assert!(f.velocity.0 < 0.0, "x velocity should invert to negative");
    }

    #[test]
    fn render_clips_left() {
        let assets = mk_assets();
        let state = AquariumState {
            size: (4, 1),
            fishes: vec![FishInstance {
                fish_art_index: 0,
                position: (-1.0, 0.0),
                velocity: (0.0, 0.0),
            }],
        };
        let s = render_aquarium_to_string(&state, &assets);
        assert_eq!(s.len(), 4);
        // Expect only the '>' to be visible when the fish is partially off-screen to the left.
        assert!(s.starts_with('>'));
    }
}
