/*!
Crate: asciiquarium_rust

Agent Log:
- Refactored crate root to use an external `widgets` module file (src/widgets/mod.rs) for clean path resolution.
- The `widgets` module will publicly expose `asciiquarium` and `asciiquarium_assets`.
- Kept ergonomic re-exports at the crate root for common types and functions.

Next steps for agents:
- Ensure `src/widgets/mod.rs` declares:
  pub mod asciiquarium;
  pub mod asciiquarium_assets;
*/

#![forbid(unsafe_code)]

pub mod widgets;

// Re-export common items for convenience at the crate root.
pub use widgets::asciiquarium::{
    render_aquarium_to_string, update_aquarium, AquariumState, AsciiquariumTheme,
    AsciiquariumWidget, FishArt, FishInstance,
};
pub use widgets::asciiquarium_assets::{get_fish_assets, measure_art};
pub use widgets::get_all_fish_assets;
