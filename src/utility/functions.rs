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