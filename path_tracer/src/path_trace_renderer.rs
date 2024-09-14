use std::iter;
use eframe::CreationContext;
use eframe::emath::Vec2;
use egui::{Image, Response, Sense, Ui};
use egui::load::SizedTexture;
use egui_wgpu::RenderState;
use wgpu::CommandEncoderDescriptor;
use crate::display_texture_pipeline::DisplayTexture;
use crate::path_tracer_package::PathTracerPackage;

pub struct PathTracerRenderer {
   path_tracer_package: PathTracerPackage,
   display_texture: DisplayTexture,
}
impl PathTracerRenderer {

   pub fn new(cc: &CreationContext) -> Self {
      let render_state = cc.wgpu_render_state.as_ref().unwrap();

      let path_tracer_package = PathTracerPackage::new(render_state);
      let display_texture = DisplayTexture::new(render_state, path_tracer_package.storage_textures.read_layout());

      Self {
         path_tracer_package,
         display_texture,
      }
   }


   pub fn update(&mut self, render_state: &RenderState) {

      self.render_pass(render_state);
   }


   pub fn display(&mut self, ui: &mut Ui) {
      let max = ui.available_size();

      // if /*render_settings.maintain_aspect_ratio*/ true {
      //    let aspect = {
      //       let w = render_settings.width;
      //       let h = render_settings.height;
      //       h as f32 / w as f32
      //    };
      //
      //    ms.height = (ms.width as f32 * aspect) as u32;
      //
      //    if ms.height > max.height {
      //       let diff = ms.height as f32 - max.height as f32;
      //
      //       ms.height -= diff as u32;
      //       ms.width -= diff as u32;
      //    }
      // }


      let response = ui.add(
         Image::from_texture(
            SizedTexture::new(
               self.display_texture.texture.texture_id,
               Vec2::new(
                  self.display_texture.texture.texture.size().width as f32,
                  self.display_texture.texture.texture.size().height as f32,
               ),
            )
         ).sense(Sense::click_and_drag())
      );
   }

   fn handle_input(&mut self, _response: Response) {}

   fn render_pass(&mut self, render_state: &RenderState) {
      let mut encoder = render_state.device.create_command_encoder(&CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         self.path_tracer_package.render_pass(&mut encoder);
         self.display_texture.render_pass(&mut encoder, &self.path_tracer_package.storage_textures.textures.item_one().read_bind_group);
      }

      render_state.queue.submit(iter::once(encoder.finish()));
   }
}