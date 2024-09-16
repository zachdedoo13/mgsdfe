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

   queue_pipeline_remake: bool, // temp
}
impl PathTracerRenderer {

   pub fn new(cc: &CreationContext) -> Self {
      let render_state = cc.wgpu_render_state.as_ref().unwrap();

      let path_tracer_package = PathTracerPackage::new(render_state);
      let display_texture = DisplayTexture::new(render_state, path_tracer_package.storage_textures.read_layout());

      Self {
         path_tracer_package,
         display_texture,
         queue_pipeline_remake: false,
      }
   }


   pub fn update(&mut self, render_state: &RenderState) {
      self.render_pass(render_state);

      if self.queue_pipeline_remake {
         self.path_tracer_package.remake_pipeline(&render_state.device);
         self.queue_pipeline_remake = false;
      }
   }


   pub fn display(&mut self, ui: &mut Ui) {
      let _max = ui.available_size();

      let _response = ui.add(
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

      if ui.button("Remake pipeline").clicked() {
         self.queue_pipeline_remake = true;
      }
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