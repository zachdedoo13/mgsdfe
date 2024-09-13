pub mod app;
pub use app::MgsApp;

pub mod ui;

#[cfg(target_arch = "wasm32")]
pub mod main;
