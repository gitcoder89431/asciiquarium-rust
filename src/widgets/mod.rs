/*!
Widgets module for the Asciiquarium crate.

Agent Log:
- Created `widgets/mod.rs` to publicly expose submodules.
- Exposes:
  - `asciiquarium`: core widget, state, update, and render logic.
  - `asciiquarium_assets`: fish ASCII assets and measurement utilities.
  - `generated_fish_assets`: auto-generated ASCII fish extracted from the original.
*/

pub mod asciiquarium;
pub mod asciiquarium_assets;
pub mod generated_fish_assets;

pub use asciiquarium_assets::get_fish_assets;
pub use generated_fish_assets::get_generated_fish_assets;

/// Return all fish assets (manual + extracted from original).
pub fn get_all_fish_assets() -> Vec<asciiquarium::FishArt> {
    let mut v = get_fish_assets();
    v.extend(get_generated_fish_assets());
    v
}
