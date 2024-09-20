#![allow(special_module_name)] // to stop main as library warning

pub mod app;
pub use app::MgsApp;

pub mod user_interface {
   pub mod ui;
}


#[cfg(target_arch = "wasm32")]
pub mod main;
