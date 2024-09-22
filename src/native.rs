use std::sync::Arc;
use log::error;

use wgpu::{Adapter, DeviceDescriptor, Features, Limits, PresentMode};

/// Native
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
   use egui_wgpu::WgpuConfiguration;
   use mgsdfe::MgsApp;

   println!("remember to hide console in releases");

   env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

   let device_descriptor_fn: Arc<dyn Fn(&Adapter) -> DeviceDescriptor<'static>> = Arc::new(|_adapter: &Adapter| {
      DeviceDescriptor {
         label: Some("wgpu native device desc"),
         required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES |
             Features::TIMESTAMP_QUERY_INSIDE_PASSES |
             Features::TIMESTAMP_QUERY |
             Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
         required_limits: Limits::default(),
      }
   });

   let native_options = eframe::NativeOptions {
      vsync: false,
      wgpu_options: WgpuConfiguration {
         present_mode: PresentMode::Immediate,
         power_preference: wgpu::PowerPreference::HighPerformance,
         device_descriptor: device_descriptor_fn,
         ..Default::default()
      },
      viewport: egui::ViewportBuilder::default()
          .with_inner_size([400.0, 300.0])
          .with_min_inner_size([300.0, 220.0]),
      ..Default::default()
   };

   eframe::run_native(
      "eframe template",
      native_options,
      Box::new(|cc| Ok(Box::new(MgsApp::new(cc)))),
   )
}