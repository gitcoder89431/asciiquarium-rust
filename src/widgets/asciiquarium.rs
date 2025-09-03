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

const CLASSIC_BUBBLE_TICKS: u64 = 24;
const CLASSIC_DT: f32 = 0.033;
const CLASSIC_FISH_SPEED_MULT: f32 = 2.0;

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

/// A surface ship moving along the waterline.
#[derive(Debug, Clone)]
pub struct Ship {
    pub x: f32,
    pub y: usize,
    pub vx: f32,
}

/// A shark swimming under water.
#[derive(Debug, Clone)]
pub struct Shark {
    pub x: f32,
    pub y: usize,
    pub vx: f32,
}

/// A whale swimming under water (with a spout animation).
#[derive(Debug, Clone)]
pub struct Whale {
    pub x: f32,
    pub y: usize,
    pub vx: f32,
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
    /// Surface ships.
    pub ships: Vec<Ship>,
    /// Underwater sharks.
    pub sharks: Vec<Shark>,
    /// Underwater whales.
    pub whales: Vec<Whale>,
    /// Next eligible tick to spawn a ship/shark/whale when none present.
    pub next_ship_spawn: u64,
    pub next_shark_spawn: u64,
    pub next_whale_spawn: u64,
    /// Next eligible tick to spawn a school of fish
    pub next_school_spawn: u64,
}

impl Default for AquariumEnvironment {
    fn default() -> Self {
        Self {
            water_phase: 0,
            seaweed: Vec::new(),
            castle: true,
            ships: Vec::new(),
            sharks: Vec::new(),
            whales: Vec::new(),
            next_ship_spawn: 0,
            next_shark_spawn: 0,
            next_whale_spawn: 0,
            next_school_spawn: 0,
        }
    }
}

/// How a fish should behave within the aquarium.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishBehavior {
    /// Normal aquarium fish: bounces on edges and persists.
    Normal,
    /// Transit fish: swims straight across and despawns once fully off-screen.
    Transit,
}

/// The aquarium state that the parent application owns and updates.
#[derive(Debug, Default)]
pub struct AquariumState {
    /// Bounds of the aquarium in character cells (width, height).
    pub size: (usize, usize),
    /// All fish currently in the aquarium.
    pub fishes: Vec<FishInstance>,
    /// Behavior associated with each fish (parallel to `fishes`). Defaults to Normal.
    pub fish_behaviors: Vec<FishBehavior>,
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

// Ships (left/right)
const SHIP_R: &str = r#"
     |    |    |
    )_)  )_)  )_)
   )___))___))___)\
  )____)____)_____)\\\
_____|____|____|____\\\\\__
\                   /
"#;

const SHIP_L: &str = r#"
         |    |    |
        (_(  (_(  (_(
      /(___((___((___(
    //(_____(____(____(
__///____|____|____|_____
    \                   /
"#;

// Sharks (left/right) - simplified large ASCII
const SHARK_R: &str = r#"
                              __
                             ( `\
  ,??????????????????????????)   `\
;' `.????????????????????????(     `\__
 ;   `.?????????????__..---''          `~~~~-._
  `.   `.____...--''                       (b  `--._
    >                     _.-'      .((      ._     )
  .`.-`--...__         .-'     -.___.....-(|/|/|/|/'
 ;.'?????????`. ...----`.___.',,,_______......---'
 '???????????'-'
"#;

const SHARK_L: &str = r#"
                     __
                    /' )
                  /'   (??????????????????????????,
              __/'     )????????????????????????.' `;
      _.-~~~~'          ``---..__?????????????.'   ;
 _.--'  b)                       ``--...____.'   .'
(     _.      )).      `-._                     <
 `\|\|\|\|)-.....___.-     `-.         __...--'-.'.
   `---......_______,,,`.___.'----... .'?????????`.;
                                     `-`???????????`
"#;

// Whales (left/right)
const WHALE_R: &str = r#"
        .-----:
      .'       `.
,????/       (o) \
\`._/          ,__)
"#;

const WHALE_L: &str = r#"
    :-----.
  .'       `.
 / (o)       \????,
(__,          \_.'/
"#;

// Water spout frames (small)
const SPOUT_FRAMES: [&str; 7] = [
    r#"

   :
"#,
    r#"
   :
   :
"#,
    r#"
  . .
  -:-
   :
"#,
    r#"
  . .
 .-:-.
   :
"#,
    r#"
  . .
'.-:-.`
'  :  '
"#,
    r#"
 .- -.
;  :  ;
"#,
    r#"

;     ;
"#,
];

fn measure_block(art: &str) -> (usize, usize) {
    let mut w = 0usize;
    let mut h = 0usize;
    for line in art.lines() {
        w = w.max(line.chars().count());
        h += 1;
    }
    (w.max(1), h.max(1))
}

/// Mirror a single ASCII character to its horizontal counterpart.
fn mirror_char(c: char) -> char {
    match c {
        '<' => '>',
        '>' => '<',
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '/' => '\\',
        '\\' => '/',
        // Characters that are symmetric or should remain unchanged by mirroring:
        '-' | '_' | '|' | '`' | '\'' | '"' | '.' | ',' | ':' | ';' | '~' | '^' | 'o' | 'O'
        | '0' => c,
        _ => c,
    }
}

/// Mirror an ASCII line: reverse order and swap mirrored pairs.
fn mirror_ascii_line(s: &str) -> String {
    s.chars().rev().map(mirror_char).collect()
}

/// Heuristic: decide if the art prefers facing right.
/// Uses a weighted score with multiple cues to reduce false “backwards” cases:
/// - Count of '>' vs '<'
/// - Right/left-facing substring cues (e.g., \"('>\", \"o>\", \"<')\", \"<o\")
/// - Line-based cues (line ends with '>' or starts with '<')
///   Defaults to right-facing when balanced.
fn art_prefers_right(art: &str) -> bool {
    // 1) Base score from arrow counts
    let mut gt = 0i32;
    let mut lt = 0i32;
    for ch in art.chars() {
        match ch {
            '>' => gt += 1,
            '<' => lt += 1,
            _ => {}
        }
    }
    let mut score = gt - lt;

    // 2) Substring cues (weighted)
    const RIGHT_CUES: [&str; 8] = ["('>", "o>", "0>", "))>", "]>", "}>", " )>", "/>"];
    const LEFT_CUES: [&str; 8] = ["<')", "<o", "<0", "<((", "<[", "<{", "<) ", "<\\"];

    let s = art;
    for cue in RIGHT_CUES {
        // weight +2 per match
        score += 2 * s.matches(cue).count() as i32;
    }
    for cue in LEFT_CUES {
        // weight -2 per match
        score -= 2 * s.matches(cue).count() as i32;
    }

    // 3) Line-based cues: line ends with '>' => right, starts with '<' => left
    for line in art.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.ends_with('>') {
            score += 1;
        }
        if t.starts_with('<') {
            score -= 1;
        }
    }

    // Default to right on tie for stability
    score >= 0
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

        // Deterministic seeded placement based on size (no external RNG).
        let mut s: u64 = 0x9E37_79B9_7F4A_7C15u64 ^ ((w as u64) << 32) ^ (h as u64);

        let mut xs: Vec<usize> = Vec::with_capacity(target_count);
        for _ in 0..target_count {
            // Advance seed and choose x in [1, w-2] when possible
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let mut x = 1 + (s as usize % w.saturating_sub(2).max(1));

            // Avoid duplicates with a few retries
            let mut retries = 0;
            while xs.contains(&x) && retries < 4 {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                x = 1 + (s as usize % w.saturating_sub(2).max(1));
                retries += 1;
            }
            xs.push(x);

            // Height 3..6
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let height = 3 + ((s as usize) % 4);

            // Sway phase randomized but deterministic
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let sway_phase = (s as u8) & 0x1F;

            state.env.seaweed.push(Seaweed {
                x,
                height,
                sway_phase,
            });
        }

        // Sort by x for stable left-to-right rendering.
        state.env.seaweed.sort_by_key(|s| s.x);
    }
}

/// Update the aquarium by one tick with simple wall-bounce physics and environment.
pub fn update_aquarium(state: &mut AquariumState, assets: &[FishArt]) {
    let (aw, ah) = (state.size.0 as f32, state.size.1 as f32);
    let dt: f32 = CLASSIC_DT;
    let fish_speed_mult: f32 = CLASSIC_FISH_SPEED_MULT;

    // Ensure environment exists.
    ensure_environment_initialized(state);

    // Keep `fish_behaviors` in sync with `fishes` (pad with Normal, or truncate).
    if state.fish_behaviors.len() < state.fishes.len() {
        state
            .fish_behaviors
            .resize(state.fishes.len(), FishBehavior::Normal);
    } else if state.fish_behaviors.len() > state.fishes.len() {
        state.fish_behaviors.truncate(state.fishes.len());
    }

    // Integrate fish and handle bounce.

    // Spawn entities deterministically when none present and past next spawn tick.
    if state.env.ships.is_empty() && state.tick >= state.env.next_ship_spawn {
        // Alternate direction by epoch (simple deterministic scheme).
        let right = (state.tick / 900) % 2 == 0;
        let (sw, _) = if right {
            measure_block(SHIP_R)
        } else {
            measure_block(SHIP_L)
        };
        let (x, vx) = if right {
            (-(sw as f32), 6.0)
        } else {
            (state.size.0 as f32 + sw as f32, -6.0)
        };
        state.env.ships.push(Ship { x, y: 0, vx });
    }
    if state.env.sharks.is_empty() && state.tick >= state.env.next_shark_spawn {
        // Place shark at a consistent depth under waterlines.
        let (_, sh) = measure_block(SHARK_R);
        let base = 9;
        let y = state.size.1.saturating_sub(sh + 3).max(base);
        let right = (state.tick / 1200) % 2 == 0;
        let (sw, _) = if right {
            measure_block(SHARK_R)
        } else {
            measure_block(SHARK_L)
        };
        let (x, vx) = if right {
            (-(sw as f32), 8.0)
        } else {
            (state.size.0 as f32 + sw as f32, -8.0)
        };
        state.env.sharks.push(Shark { x, y, vx });
    }
    if state.env.whales.is_empty() && state.tick >= state.env.next_whale_spawn {
        // Mid-depth whale.
        let y = (state.size.1 / 3).max(6);
        let right = (state.tick / 1500) % 2 == 0;
        let (ww, _) = if right {
            measure_block(WHALE_R)
        } else {
            measure_block(WHALE_L)
        };
        let (x, vx) = if right {
            (-(ww as f32), 4.0)
        } else {
            (state.size.0 as f32 + ww as f32, -4.0)
        };
        state.env.whales.push(Whale { x, y, vx });
    }
    // Spawn a deterministic school of fish at intervals
    if state.tick >= state.env.next_school_spawn && !assets.is_empty() {
        let assets_len = assets.len();
        let dir_right = (state.tick / 2400) % 2 == 0;
        let count = 5 + ((state.tick as usize / 7) % 6); // 5..10
        let mid = (state.size.1 / 2).max(6);
        let y = mid.saturating_sub(2) + ((state.tick as usize / 11) % 4);
        let speed = if dir_right { 8.0 } else { -8.0 };
        let start_x = if dir_right {
            -12.0
        } else {
            state.size.0 as f32 + 12.0
        };
        // Stagger fish along entry to keep tight school
        for i in 0..count {
            let art_idx = (i + ((state.tick as usize / 5) % assets_len)) % assets_len;
            let xi = start_x - (i as f32) * 4.0;
            state.fishes.push(FishInstance {
                fish_art_index: art_idx,
                position: (xi, y as f32),
                velocity: (speed, 0.0),
            });
            state.fish_behaviors.push(FishBehavior::Transit);
        }
        state.env.next_school_spawn = state.tick + 1800; // ~60s at 30 fps
    }
    // Update fish with behavior-aware logic (Transit vs Normal).
    let mut kept_fishes: Vec<FishInstance> = Vec::with_capacity(state.fishes.len());
    let mut kept_behaviors: Vec<FishBehavior> = Vec::with_capacity(state.fish_behaviors.len());
    for i in 0..state.fishes.len() {
        let mut fish = state.fishes[i].clone();
        let behavior = *state.fish_behaviors.get(i).unwrap_or(&FishBehavior::Normal);

        fish.position.0 += fish.velocity.0 * dt * fish_speed_mult;
        fish.position.1 += fish.velocity.1 * dt * fish_speed_mult;
        // Subtle deterministic horizontal jitter (does not mutate velocity)
        let jitter_seed = ((state.tick as u32).wrapping_mul(1103515245))
            ^ (fish.fish_art_index as u32).wrapping_mul(12345);
        let jitter = ((jitter_seed & 0x0F) as f32 - 8.0) * 0.002;
        fish.position.0 += jitter;

        let (fw, fh) = assets
            .get(fish.fish_art_index)
            .map(|a| (a.width as f32, a.height as f32))
            .unwrap_or((1.0, 1.0));

        if behavior == FishBehavior::Transit {
            // Despawn transit fish once fully off-screen.
            let off_right = fish.position.0 > aw;
            let off_left = fish.position.0 + fw <= 0.0;
            if off_right || off_left {
                // drop (do not keep)
            } else {
                kept_fishes.push(fish);
                kept_behaviors.push(behavior);
            }
            continue;
        }

        // Normal bounce behavior.
        if fish.position.0 < 0.0 {
            fish.position.0 = 0.0;
            fish.velocity.0 = fish.velocity.0.abs();
            // Small deterministic chance to flip vertical direction on wall bounce for natural variance.
            let flip_seed =
                ((state.tick) ^ ((fish.fish_art_index as u64) << 12) ^ 0x9E37_79B9_7F4A_7C15)
                    .wrapping_mul(6364136223846793005);
            if (flip_seed & 0x0F) == 0 {
                fish.velocity.1 = -fish.velocity.1;
            }
        } else if fish.position.0 + fw > aw {
            fish.position.0 = (aw - fw).max(0.0);
            fish.velocity.0 = -fish.velocity.0.abs();
            // Small deterministic chance to flip vertical direction on wall bounce for natural variance.
            let flip_seed =
                ((state.tick) ^ ((fish.fish_art_index as u64) << 13) ^ 0x9E37_79B9_7F4A_7C15)
                    .wrapping_mul(6364136223846793005);
            if (flip_seed & 0x0F) == 0 {
                fish.velocity.1 = -fish.velocity.1;
            }
        }

        if fish.position.1 < 0.0 {
            fish.position.1 = 0.0;
            fish.velocity.1 = fish.velocity.1.abs();
            // Small deterministic chance to flip horizontal direction on wall bounce for natural variance.
            let flip_seed =
                ((state.tick) ^ ((fish.fish_art_index as u64) << 14) ^ 0x9E37_79B9_7F4A_7C15)
                    .wrapping_mul(6364136223846793005);
            if (flip_seed & 0x0F) == 0 {
                fish.velocity.0 = -fish.velocity.0;
            }
        } else if fish.position.1 + fh > ah {
            fish.position.1 = (ah - fh).max(0.0);
            fish.velocity.1 = -fish.velocity.1.abs();
            // Small deterministic chance to flip horizontal direction on wall bounce for natural variance.
            let flip_seed =
                ((state.tick) ^ ((fish.fish_art_index as u64) << 15) ^ 0x9E37_79B9_7F4A_7C15)
                    .wrapping_mul(6364136223846793005);
            if (flip_seed & 0x0F) == 0 {
                fish.velocity.0 = -fish.velocity.0;
            }
        }

        kept_fishes.push(fish);
        kept_behaviors.push(behavior);
    }
    state.fishes = kept_fishes;
    state.fish_behaviors = kept_behaviors;

    // Occasionally emit bubbles from fish mouths, deterministically based on tick.
    // Emit every 24 ticks per fish to avoid randomness in the core crate.
    for (i, fish) in state.fishes.iter().enumerate() {
        // Desync bubble emission per fish using deterministic staggering
        if (state.tick + ((i as u64 * 11) % CLASSIC_BUBBLE_TICKS)) % CLASSIC_BUBBLE_TICKS == 0 {
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
                velocity: (0.0, -3.0),
            });
        }
    }

    // Update bubbles (rise) and cull above waterline (y < 0).
    let mut kept = Vec::with_capacity(state.bubbles.len());
    for mut b in state.bubbles.drain(..) {
        b.position.0 += b.velocity.0 * dt;
        b.position.1 += b.velocity.1 * dt;
        if b.position.1 >= 0.0 {
            kept.push(b);
        }
    }
    state.bubbles = kept;

    // Move ships and despawn when fully off-screen. Schedule next spawn.
    let mut next_ships = Vec::with_capacity(state.env.ships.len());
    for mut ship in state.env.ships.drain(..) {
        ship.x += ship.vx * dt;
        let (sw, _) = if ship.vx >= 0.0 {
            measure_block(SHIP_R)
        } else {
            measure_block(SHIP_L)
        };
        let off_right = ship.x > state.size.0 as f32;
        let off_left = ship.x + sw as f32 <= 0.0;
        if off_right || off_left {
            // Next ship after ~20s
            state.env.next_ship_spawn = state.tick + 600;
        } else {
            next_ships.push(ship);
        }
    }
    state.env.ships = next_ships;

    // Move sharks and despawn when fully off-screen. Schedule next spawn.
    let mut next_sharks = Vec::with_capacity(state.env.sharks.len());
    for mut shark in state.env.sharks.drain(..) {
        shark.x += shark.vx * dt;
        let (sw, _) = if shark.vx >= 0.0 {
            measure_block(SHARK_R)
        } else {
            measure_block(SHARK_L)
        };
        let off_right = shark.x > state.size.0 as f32;
        let off_left = shark.x + sw as f32 <= 0.0;
        if off_right || off_left {
            // Next shark after ~30s
            state.env.next_shark_spawn = state.tick + 900;
        } else {
            next_sharks.push(shark);
        }
    }
    state.env.sharks = next_sharks;

    // Move whales and despawn when fully off-screen. Schedule next spawn.
    let mut next_whales = Vec::with_capacity(state.env.whales.len());
    for mut whale in state.env.whales.drain(..) {
        whale.x += whale.vx * dt;
        let (ww, _) = if whale.vx >= 0.0 {
            measure_block(WHALE_R)
        } else {
            measure_block(WHALE_L)
        };
        let off_right = whale.x > state.size.0 as f32;
        let off_left = whale.x + ww as f32 <= 0.0;
        if off_right || off_left {
            // Next whale after ~40s
            state.env.next_whale_spawn = state.tick + 1200;
        } else {
            next_whales.push(whale);
        }
    }
    state.env.whales = next_whales;

    // Advance environment phases.
    if state.tick % 4 == 0 {
        state.env.water_phase = state.env.water_phase.wrapping_add(1);
    }
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

    // 1) Waterlines with per-column vertical offsets for wave dynamics.
    let patterns: [Vec<char>; 4] = [
        WATER_LINES[0].chars().collect(),
        WATER_LINES[1].chars().collect(),
        WATER_LINES[2].chars().collect(),
        WATER_LINES[3].chars().collect(),
    ];
    let plens = [
        patterns[0].len().max(1),
        patterns[1].len().max(1),
        patterns[2].len().max(1),
        patterns[3].len().max(1),
    ];

    for x in 0..w {
        // Triangular wave over columns with phase: 0 -> 1 -> 2 -> 1 repeating.
        let t = (state.env.water_phase as usize + x) % 24;
        let v_off: usize = if t < 6 {
            0
        } else if t < 12 {
            1
        } else if t < 18 {
            2
        } else {
            1
        };

        for i in 0..4 {
            if i >= h {
                break;
            }
            let y = i + v_off;
            if y >= h {
                continue;
            }
            let off = (state.env.water_phase as usize) % plens[i];
            let ch = patterns[i][(x + off) % plens[i]];
            grid[y * w + x] = ch;
        }
    }

    // Render ships over waterlines near the surface.
    for ship in &state.env.ships {
        let x0 = ship.x.floor() as isize;
        // Subtle vertical bob (amplitude 1) with slow phase
        let phase_u = (state.tick / 64) % 2;
        let phase_i = (x0.rem_euclid(2)) as u64;
        let bob = if (phase_u + phase_i) % 2 == 0 { 0 } else { 1 };
        let y0 = ship.y as isize + bob;
        let art = if ship.vx >= 0.0 { SHIP_R } else { SHIP_L };
        for (dy, line) in art.lines().enumerate() {
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

    // Render whales (with spout) and sharks under water.
    for whale in &state.env.whales {
        let x0 = whale.x.floor() as isize;
        // Subtle vertical bob (amplitude 1) with slow phase
        let phase_u = (state.tick / 96) % 2;
        let phase_i = (x0.rem_euclid(2)) as u64;
        let bob = if (phase_u + phase_i) % 2 == 0 { 0 } else { 1 };
        let y0 = whale.y as isize + bob;
        let art = if whale.vx >= 0.0 { WHALE_R } else { WHALE_L };
        // Whale body
        for (dy, line) in art.lines().enumerate() {
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
        // Water spout above head (simple animation)
        let frame = (state.tick as usize / 12) % SPOUT_FRAMES.len();
        let spout = SPOUT_FRAMES[frame];
        // Approximate blowhole position a bit right of whale x
        let spx = x0 + if whale.vx >= 0.0 { 8 } else { 3 };
        let spy = y0.saturating_sub(3);
        for (dy, line) in spout.lines().enumerate() {
            let y = spy + dy as isize;
            if y < 0 || y >= h as isize {
                continue;
            }
            for (dx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = spx + dx as isize;
                if x < 0 || x >= w as isize {
                    continue;
                }
                grid[y as usize * w + x as usize] = ch;
            }
        }
    }

    for shark in &state.env.sharks {
        let x0 = shark.x.floor() as isize;
        // Subtle vertical bob (amplitude 1) with slow phase
        let phase_u = (state.tick / 80) % 2;
        let phase_i = (x0.rem_euclid(2)) as u64;
        let bob = if (phase_u + phase_i) % 2 == 0 { 0 } else { 1 };
        let y0 = shark.y as isize + bob;
        let art = if shark.vx >= 0.0 { SHARK_R } else { SHARK_L };
        for (dy, line) in art.lines().enumerate() {
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

    // 4) Fish (overdraw seaweed/castle/water where they overlap).
    for fish in &state.fishes {
        let art = match assets.get(fish.fish_art_index) {
            Some(a) => a,
            None => continue,
        };
        let x0 = fish.position.0.floor() as isize;
        let y0 = fish.position.1.floor() as isize;

        let mirror = {
            let prefers_right = art_prefers_right(art.art);
            (fish.velocity.0 < 0.0 && prefers_right) || (fish.velocity.0 > 0.0 && !prefers_right)
        };
        for (dy, raw_line) in art.art.lines().enumerate() {
            let y = y0 + dy as isize;
            if y < 0 || y >= h as isize {
                continue;
            }

            let line_str = if mirror {
                mirror_ascii_line(raw_line)
            } else {
                raw_line.to_string()
            };

            for (dx, ch) in line_str.chars().enumerate() {
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
