/*!
Asciiquarium widget scaffold for egui.

Agent Log:
- Extended AquariumState with environment (waterlines, seaweed, castle) and bubbles, plus a tick counter.
- Implemented environment initialization and simple wave/seaweed animation phases.
- Added bubble emission from fish and upward drift with culling at waterline.
- Updated rendering to draw waterlines, castle, seaweed, fishes, then bubbles (top-most).
- Preserved stateless widget and single-label rendering approach.
- Kept bounce physics and clipping; float-to-int via floor() for stability.
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

/// A bubble that rises towards the waterline.
#[derive(Debug, Clone)]
pub struct Bubble {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
}

/// A single seaweed stalk.
#[derive(Debug, Clone)]
pub struct Seaweed {
    pub x: usize,
    pub height: usize,
    /// Per-stalk phase to desynchronize sway animation.
    pub sway_phase: u8,
}

/// Environment effects and static props.
#[derive(Debug, Clone)]
pub struct AquariumEnvironment {
    /// Phase for waterline horizontal offset/sway animations.
    pub water_phase: u8,
    /// Detected/generated set of seaweed stalks.
    pub seaweed: Vec<Seaweed>,
    /// Whether to render the castle at bottom-right.
    pub castle: bool,
}

impl Default for AquariumEnvironment {
    fn default() -> Self {
        Self {
            water_phase: 0,
            seaweed: Vec::new(),
            castle: true,
        }
    }
}

/// The aquarium state that the parent application owns and updates.
#[derive(Debug, Default)]
pub struct AquariumState {
    /// Bounds of the aquarium in character cells (width, height).
    pub size: (usize, usize),
    /// All fish currently in the aquarium.
    pub fishes: Vec<FishInstance>,
    /// Rising bubbles.
    pub bubbles: Vec<Bubble>,
    /// Background/props animation state.
    pub env: AquariumEnvironment,
    /// Tick counter advanced once per update.
    pub tick: u64,
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

// Static environment art and helpers.

const WATER_LINES: [&str; 4] = [
    "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
    "^^^^ ^^^  ^^^   ^^^    ^^^^      ",
    "^^^^      ^^^^     ^^^    ^^     ",
    "^^      ^^^^      ^^^    ^^^^^^  ",
];

const CASTLE: &str = r#"
               T~~
               |
              /^\
             /   \
 _   _   _  /     \  _   _   _
[ ]_[ ]_[ ]/ _   _ \[ ]_[ ]_[ ]
|_=__-_ =_|_[ ]_[ ]_|_=-___-__|
 | _- =  | =_ = _    |= _=   |
 |= -[]  |- = _ =    |_-=_[] |
 | =_    |= - ___    | =_ =  |
 |=  []- |-  /| |\   |=_ =[] |
 |- =_   | =| | | |  |- = -  |
 |_______|__|_|_|_|__|_______|
"#;

fn measure_block(art: &str) -> (usize, usize) {
    let mut w = 0usize;
    let mut h = 0usize;
    for line in art.lines() {
        w = w.max(line.chars().count());
        h += 1;
    }
    (w.max(1), h.max(1))
}

fn ensure_environment_initialized(state: &mut AquariumState) {
    // Generate seaweed based on width if none present or if size changed significantly.
    let (w, h) = state.size;
    if w == 0 || h == 0 {
        state.env.seaweed.clear();
        return;
    }
    let target_count = (w / 15).max(1);
    if state.env.seaweed.len() != target_count {
        state.env.seaweed.clear();
        // Evenly distribute stalks across width; deterministic heights.
        for i in 0..target_count {
            let x = ((i + 1) * w / (target_count + 1)).saturating_sub(1);
            let height = 3 + (i % 4); // 3..6
            state.env.seaweed.push(Seaweed {
                x,
                height,
                sway_phase: (i as u8) * 7,
            });
        }
    }
}

/// Update the aquarium by one tick with simple wall-bounce physics and environment.
pub fn update_aquarium(state: &mut AquariumState, assets: &[FishArt]) {
    let (aw, ah) = (state.size.0 as f32, state.size.1 as f32);

    // Ensure environment exists.
    ensure_environment_initialized(state);

    // Integrate fish and handle bounce.
    for fish in &mut state.fishes {
        fish.position.0 += fish.velocity.0;
        fish.position.1 += fish.velocity.1;

        let (fw, fh) = assets
            .get(fish.fish_art_index)
            .map(|a| (a.width as f32, a.height as f32))
            .unwrap_or((1.0, 1.0));

        if fish.position.0 < 0.0 {
            fish.position.0 = 0.0;
            fish.velocity.0 = fish.velocity.0.abs();
        } else if fish.position.0 + fw > aw {
            fish.position.0 = (aw - fw).max(0.0);
            fish.velocity.0 = -fish.velocity.0.abs();
        }

        if fish.position.1 < 0.0 {
            fish.position.1 = 0.0;
            fish.velocity.1 = fish.velocity.1.abs();
        } else if fish.position.1 + fh > ah {
            fish.position.1 = (ah - fh).max(0.0);
            fish.velocity.1 = -fish.velocity.1.abs();
        }
    }

    // Occasionally emit bubbles from fish mouths, deterministically based on tick.
    // Emit every 24 ticks per fish to avoid randomness in the core crate.
    for fish in &state.fishes {
        if state.tick % 24 == 0 {
            let (fw, fh) = assets
                .get(fish.fish_art_index)
                .map(|a| (a.width as f32, a.height as f32))
                .unwrap_or((1.0, 1.0));

            let mid_y = fish.position.1 + fh * 0.5;
            let bx = if fish.velocity.0 >= 0.0 {
                fish.position.0 + fw
            } else {
                fish.position.0 - 1.0
            };
            state.bubbles.push(Bubble {
                position: (bx, mid_y),
                velocity: (0.0, -0.3),
            });
        }
    }

    // Update bubbles (rise) and cull above waterline (y < 0).
    let mut kept = Vec::with_capacity(state.bubbles.len());
    for mut b in state.bubbles.drain(..) {
        b.position.0 += b.velocity.0;
        b.position.1 += b.velocity.1;
        if b.position.1 >= 0.0 {
            kept.push(b);
        }
    }
    state.bubbles = kept;

    // Advance environment phases.
    state.env.water_phase = state.env.water_phase.wrapping_add(1);
    state.tick = state.tick.wrapping_add(1);
}

/// Render the aquarium state into a single string (newline-separated).
///
/// Order:
/// - Waterlines (background)
/// - Castle (bottom-right)
/// - Seaweed (foreground under fish)
/// - Fish
/// - Bubbles (top-most)
pub fn render_aquarium_to_string(state: &AquariumState, assets: &[FishArt]) -> String {
    let (w, h) = state.size;
    if w == 0 || h == 0 {
        return String::new();
    }

    let mut grid = vec![' '; w * h];

    // 1) Waterlines (top 4 rows), animated horizontal offset by water_phase.
    for (i, pattern) in WATER_LINES.iter().enumerate() {
        if i >= h {
            break;
        }
        let chars: Vec<char> = pattern.chars().collect();
        let plen = chars.len().max(1);
        let offset = (state.env.water_phase as usize) % plen;
        for x in 0..w {
            let ch = chars[(x + offset) % plen];
            let idx = i * w + x;
            grid[idx] = ch;
        }
    }

    // 2) Castle at bottom-right if enabled.
    if state.env.castle {
        let (cw, ch) = measure_block(CASTLE);
        let base_x = w.saturating_sub(cw + 1);
        let base_y = h.saturating_sub(ch);
        for (dy, line) in CASTLE.lines().enumerate() {
            let y = base_y + dy;
            if y >= h {
                continue;
            }
            for (dx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = base_x + dx;
                if x >= w {
                    continue;
                }
                grid[y * w + x] = ch;
            }
        }
    }

    // 3) Seaweed stalks, swaying slightly with water_phase + per-stalk phase.
    for (idx, stalk) in state.env.seaweed.iter().enumerate() {
        let base_y = h.saturating_sub(stalk.height);
        // sway: -1, 0, +1 cycling at a slow rate
        let phase = (state.env.water_phase.wrapping_add(stalk.sway_phase)) / 8;
        let sway = match phase % 3 {
            0 => -1isize,
            1 => 0isize,
            _ => 1isize,
        };
        // Draw alternating '(' and ')' vertically.
        for dy in 0..stalk.height {
            let y = base_y + dy;
            if y >= h {
                continue;
            }
            let left = dy % 2 == 0;
            let x_base = stalk.x as isize + if left { 0 } else { 1 };
            let x = x_base + sway;
            if x < 0 || (x as usize) >= w {
                continue;
            }
            grid[y * w + (x as usize)] = if left { '(' } else { ')' };
        }
        // Slight horizontal spread for some stalks to avoid uniformity.
        if idx % 3 == 0 {
            let x2 = (stalk.x + 1).min(w.saturating_sub(1));
            for dy in 1..stalk.height {
                let y = base_y + dy;
                if y >= h {
                    continue;
                }
                let x = x2 as isize + sway;
                if x < 0 || (x as usize) >= w {
                    continue;
                }
                if dy % 2 == 0 {
                    grid[y * w + (x as usize)] = '(';
                } else {
                    grid[y * w + (x as usize)] = ')';
                }
            }
        }
    }

    // 4) Fish (overdraw seaweed/castle/water where they overlap).
    for fish in &state.fishes {
        let art = match assets.get(fish.fish_art_index) {
            Some(a) => a,
            None => continue,
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
                grid[y as usize * w + x as usize] = ch;
            }
        }
    }

    // 5) Bubbles (top-most), simple '.' markers with clipping.
    for b in &state.bubbles {
        let x = b.position.0.floor() as isize;
        let y = b.position.1.floor() as isize;
        if x < 0 || x >= w as isize || y < 0 || y >= h as isize {
            continue;
        }
        grid[y as usize * w + x as usize] = '.';
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
            ..Default::default()
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
            size: (4, 5), // leave room for waterlines
            fishes: vec![FishInstance {
                fish_art_index: 0,
                position: (-1.0, 0.0),
                velocity: (0.0, 0.0),
            }],
            ..Default::default()
        };
        let s = render_aquarium_to_string(&state, &assets);
        // Expect multiple rows; ensure first visible char is still fish '>' due to overdraw.
        assert!(s.lines().next().unwrap_or("").starts_with('>'));
    }
}
