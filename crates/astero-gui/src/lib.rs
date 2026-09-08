//! Independent host GUI. Session APIs remain toolkit-independent.
mod app;
pub mod model;
mod renderer;
mod ui;
mod window;
pub use app::run;
// Preserve the existing M1 adapter path.
pub use model as view_model;
