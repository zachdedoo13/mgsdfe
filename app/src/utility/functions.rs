use std::f32::consts::PI;

use eframe::wgpu::Extent3d;
use egui::Vec2;

pub fn to_v2(extent: Extent3d) -> Vec2 {
   Vec2::new(extent.width as f32, extent.height as f32)
}

pub fn to_extent(vec2: Vec2) -> Extent3d {
   Extent3d {
      width: vec2.x as u32,
      height: vec2.y as u32,
      depth_or_array_layers: 1,
   }
}



pub fn oss(reference: f64, freq: f64, amp: f64, phase: f64) -> f64 {
   amp * (2.0f64 * PI as f64 * freq * reference + phase).sin()
}