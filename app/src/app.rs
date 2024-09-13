use std::time::Duration;

use eframe::{App, CreationContext, Frame, Storage};
use eframe::epaint::Rgba;
use egui::{CentralPanel, Context, Visuals};
use egui_wgpu::RenderState;

use common::get;
use common::singletons::time_package::TIME;
use graph_editor::GraphEditor;
use path_tracer::PathTracer;

use crate::ui::UiState;

pub struct MgsApp {
   pub path_tracer: PathTracer,
   pub graph_editor: GraphEditor,

   pub ui_state: UiState,
}

/// main functions
impl MgsApp {
   pub fn new(_cc: &CreationContext) -> Self {
      Self {
         path_tracer: PathTracer {},
         graph_editor: GraphEditor {},

         ui_state: UiState::default(),
      }
   }

   /// global update inter-loop
   pub fn update(&mut self, _render_state: &RenderState) {
      // update singletons
      get!(TIME).update();
   }
}


/// eframe shizz
impl App for MgsApp {
   fn update(&mut self, ctx: &Context, frame: &mut Frame) {
      self.update(frame.wgpu_render_state().unwrap());

      // overload panel
      CentralPanel::default()
          .show(ctx, |ui| {
             self.ui(ui);
          });

      ctx.request_repaint();
   }

   fn save(&mut self, _storage: &mut dyn Storage) {}

   fn on_exit(&mut self) {}

   fn auto_save_interval(&self) -> Duration {
      Duration::from_secs(10)
   }

   fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
      Rgba::BLACK.to_array()
   }
}