#![warn(clippy::all, rust_2018_idioms)]
#![allow(special_module_name)]

#[cfg(target_arch = "wasm32")]
pub mod main;

pub use app::MehApp;
pub mod app;

pub mod render_state {
   pub mod structs;
   pub mod vertex_library;
   pub mod vertex_package;
   pub mod meh_renderer;
   pub mod test {
      pub mod test_render_pipeline;
   }
}

pub mod packages {
   pub mod time_package;
}

pub mod utility {
   pub mod macros;
   pub mod functions;
}