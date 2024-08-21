use std::iter;
use eframe::egui_wgpu::Renderer;
use eframe::wgpu::{CommandEncoderDescriptor, Device, Extent3d, TextureFormat};
use egui::load::SizedTexture;
use egui::{Image, Ui, Vec2};
use crate::render_state::structs::{EguiTexturePackage};
use crate::render_state::test::test_render_pipeline::TestRenderPipeline;
use crate::utility::functions::to_extent;
use crate::render_state::structs::RenderPack;

pub struct MehRenderer {
   pub test_render_pipeline: TestRenderPipeline,
   pub egui_texture_package: EguiTexturePackage,
}
impl MehRenderer {
   pub fn new(device: &Device, renderer: &mut Renderer) -> Self {
      let test_render_pipeline = TestRenderPipeline::new(device, TextureFormat::Rgba8Unorm);

      let egui_texture_package = EguiTexturePackage::new(Extent3d {
         width: 250,
         height: 250,
         depth_or_array_layers: 1,
      }, device, renderer);

      Self {
         test_render_pipeline,
         egui_texture_package,
      }
   }

   pub fn update(&mut self, render_pack: &mut RenderPack<'_>) {
      self.egui_texture_package.update(render_pack);
   }

   pub fn render_pass(&self, render_pack: &RenderPack<'_>) {
      let mut encoder = render_pack.device.create_command_encoder(&CommandEncoderDescriptor {
         label: Some("the only encoder"),
      });

      self.test_render_pipeline.render_pass(&mut encoder, &self.egui_texture_package.view);


      render_pack.queue.submit(iter::once(encoder.finish()));
   }

   pub fn display(&mut self, ui: &mut Ui) {
      let ms = to_extent(ui.available_size());
      self.egui_texture_package.size = ms;

      ui.add(
         Image::from_texture(
            SizedTexture::new(
               self.egui_texture_package.texture_id,
               Vec2::new(
                  self.egui_texture_package.texture.size().width as f32,
                  self.egui_texture_package.texture.size().height as f32
               )
            )
         )
      );
   }
}