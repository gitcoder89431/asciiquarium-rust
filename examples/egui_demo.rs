use std::time::Duration;

use asciiquarium_rust::{
    get_all_fish_assets, update_aquarium, AquariumState, AsciiquariumTheme, AsciiquariumWidget,
    FishInstance,
};
use eframe::egui;
use rand::Rng;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Asciiquarium egui demo",
        native_options,
        Box::new(|_cc| Box::new(MyApp::new())),
    )
}

struct MyApp {
    assets: Vec<asciiquarium_rust::FishArt>,
    state: AquariumState,
    theme: AsciiquariumTheme,
    frame_ms: u64,
    bg_enabled: bool,
}

impl MyApp {
    fn new() -> Self {
        let assets = get_all_fish_assets();

        // Choose an initial grid size. You can make this dynamic later if desired.
        let size = (80usize, 24usize);

        let mut state = AquariumState {
            size,
            fishes: Vec::new(),
        };

        // Seed with a few random fish
        for _ in 0..6 {
            spawn_random_fish(&mut state, assets.len());
        }

        let theme = AsciiquariumTheme {
            text_color: egui::Color32::from_rgb(180, 220, 255),
            background: Some(egui::Color32::from_rgb(8, 12, 16)),
            wrap: false,
        };

        Self {
            assets,
            state,
            theme,
            frame_ms: 33,
            bg_enabled: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drive animation based on a simple frame duration.
        update_aquarium(&mut self.state, &self.assets);
        ctx.request_repaint_after(Duration::from_millis(self.frame_ms));

        egui::TopBottomPanel::top("top_controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Grid:");
                // Keep grid integers reasonable. Avoid sliders for usize to reduce friction.
                if ui.button("-W").clicked() && self.state.size.0 > 10 {
                    self.state.size.0 -= 2;
                }
                if ui.button("+W").clicked() {
                    self.state.size.0 += 2;
                }
                if ui.button("-H").clicked() && self.state.size.1 > 5 {
                    self.state.size.1 -= 1;
                }
                if ui.button("+H").clicked() {
                    self.state.size.1 += 1;
                }

                ui.separator();

                ui.label("Frame (ms):");
                if ui.button("-").clicked() && self.frame_ms > 5 {
                    self.frame_ms -= 2;
                }
                if ui.button("+").clicked() && self.frame_ms < 1000 {
                    self.frame_ms += 2;
                }

                ui.separator();

                ui.label("Theme:");
                ui.color_edit_button_srgba(&mut self.theme.text_color);
                ui.checkbox(&mut self.bg_enabled, "Background");
                if self.bg_enabled {
                    // Ensure background stays Some when enabled
                    if self.theme.background.is_none() {
                        self.theme.background = Some(egui::Color32::from_rgb(8, 12, 16));
                    }
                    if let Some(bg) = &mut self.theme.background {
                        ui.color_edit_button_srgba(bg);
                    }
                } else {
                    self.theme.background = None;
                }

                ui.separator();

                if ui.button("Add fish").clicked() {
                    spawn_random_fish(&mut self.state, self.assets.len());
                }
                if ui.button("Reset").clicked() {
                    self.state.fishes.clear();
                    for _ in 0..6 {
                        spawn_random_fish(&mut self.state, self.assets.len());
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Render widget as a single monospace label
            ui.add(AsciiquariumWidget {
                state: &self.state,
                assets: &self.assets,
                theme: &self.theme,
            });
        });
    }
}

fn spawn_random_fish(state: &mut AquariumState, asset_count: usize) {
    if asset_count == 0 {
        return;
    }
    let mut rng = rand::thread_rng();

    let idx = rng.gen_range(0..asset_count);

    // Random position within grid; update() will clamp on edges using asset size
    let max_x = if state.size.0 > 0 {
        state.size.0 - 1
    } else {
        0
    };
    let max_y = if state.size.1 > 0 {
        state.size.1 - 1
    } else {
        0
    };
    let x = rng.gen_range(0..=max_x) as f32;
    let y = rng.gen_range(0..=max_y) as f32;

    // Random velocity with minimum magnitude to avoid stationary fish
    let mut vx = rng.gen_range(-0.5_f32..=0.5_f32);
    let mut vy = rng.gen_range(-0.25_f32..=0.25_f32);
    if vx.abs() < 0.05 {
        vx = if vx.is_sign_negative() { -0.08 } else { 0.08 };
    }
    if vy.abs() < 0.02 {
        vy = if vy.is_sign_negative() { -0.03 } else { 0.03 };
    }

    state.fishes.push(FishInstance {
        fish_art_index: idx,
        position: (x, y),
        velocity: (vx, vy),
    });
}
