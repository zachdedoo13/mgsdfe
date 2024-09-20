use std::time::Duration;

use eframe::{App, CreationContext, Frame, Storage};
use eframe::epaint::Rgba;
use egui::{CentralPanel, Context, Visuals};
use egui_wgpu::RenderState;

use common::{get_mut, set_none_static};
use common::singletons::settings::{SETTINGS, Settings};
use common::singletons::time_package::TIME;
use graph_editor::GraphEditor;
use path_tracer::path_trace_renderer::PathTracerRenderer;

use crate::user_interface::ui::UiState;

pub struct MgsApp {
   pub path_tracer: PathTracerRenderer,
   pub graph_editor: GraphEditor,
   pub ui_state: UiState,

   pub restart_queued: bool,
}

/// main functions
impl MgsApp {
   pub fn new(cc: &CreationContext) -> Self {
      // init none singletons
      set_none_static!(SETTINGS => { Settings::new(cc) });

      // init packages
      let path_tracer = PathTracerRenderer::new(cc);
      let ui_state = UiState::new(cc);

      Self {
         path_tracer,
         graph_editor: GraphEditor {},
         ui_state,

         restart_queued: false,
      }
   }

   /// global update inter-loop
   pub fn update(&mut self, render_state: &RenderState) {
      // update singletons
      get_mut!(TIME).update();

      // update modules
      self.path_tracer.update(render_state);
   }

   pub fn restart(&mut self) {
      self.restart_queued = true;
   }
}


/// eframe shizz
impl App for MgsApp {
   fn update(&mut self, ctx: &Context, frame: &mut Frame) {
      self.update(frame.wgpu_render_state().expect("Failed to unwrap render state"));

      // overload panel
      CentralPanel::default()
          .show(ctx, |ui| {
             self.ui(ui);
          });

      ctx.request_repaint();

      if self.restart_queued {
         if let Some(s) = frame.storage_mut() {
            self.restart_queued = false;
            self.save(s);
            restart_app();
         }
      }
   }

   fn save(&mut self, storage: &mut dyn Storage) {
      self.graph_editor.save(storage);
      self.ui_state.save(storage);
      get_mut!(SETTINGS).save(storage);
   }

   fn on_exit(&mut self) {}

   fn auto_save_interval(&self) -> Duration {
      Duration::from_secs(5)
   }

   fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
      Rgba::BLACK.to_array()
   }
}


#[cfg(not(target_arch = "wasm32"))]
fn restart_app() {
   use std::process::Command;
   let mut cmd = Command::new(std::env::current_exe().unwrap());
   let _ = cmd.spawn();
   std::process::exit(0);
}

#[cfg(target_arch = "wasm32")]
fn restart_app() {
   use wasm_bindgen::prelude::*;
   #[wasm_bindgen]
   extern "C" {
      #[wasm_bindgen(js_namespace = location)]
      fn reload();
   }
   unsafe { reload() };
}