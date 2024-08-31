#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use std::f32::consts::PI;
use std::time::Instant;
use graphing_stuff::app::NodeGraph;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
   use eframe::egui::Visuals;

   eframe::run_native(
      "Egui node graph example",
      eframe::NativeOptions::default(),
      Box::new(|cc| {
         cc.egui_ctx.set_visuals(Visuals::dark());
         #[cfg(feature = "persistence")]
         {
            Ok(Box::new(NodeGraph::new(cc)))
         }
         #[cfg(not(feature = "persistence"))]
         Ok(Box::<NodeGraph>::default())
      }),
   )
       .expect("Failed to run native example");

}