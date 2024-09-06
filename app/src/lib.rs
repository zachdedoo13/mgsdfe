#![warn(clippy::all, rust_2018_idioms)]
#![allow(special_module_name)]

pub use app::MehApp;

#[cfg(target_arch = "wasm32")]
pub mod main;

pub mod app;

pub mod utility {
   pub mod macros;
   pub mod functions;
   pub mod structs;
}