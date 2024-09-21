#![allow(special_module_name)] // to stop main as library warning
#![allow(
   clippy::new_without_default,
   clippy::derivable_impls,
   clippy::ptr_arg,
   clippy::module_inception,
)]

pub use app::MgsApp;

pub mod app;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub mod global_utility {
   pub mod functions;
   pub mod macros;
   pub mod structs;
}

pub mod user_interface {
   pub mod ui;
   pub mod ui_modules;
}

pub mod singletons {
   pub mod scene;
   pub mod settings;
   pub mod time_package;
}

pub mod graph_editor {
   pub mod graph_editor;
}

pub mod path_tracer {
   pub mod display_texture_pipeline;
   pub mod path_tracer_package;
   pub mod path_trace_renderer;
   pub mod render_utility {
      pub mod dual_storage_texture_package;
      pub mod helper_structs;
      pub mod vertex_package;
      pub mod vertex_library;
      pub mod gpu_profiler;
   }
}
