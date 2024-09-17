use std::iter;
use eframe::CreationContext;
use eframe::emath::Vec2;
use egui::{Image, Response, Sense, Ui};
use egui::load::SizedTexture;
use egui_wgpu::RenderState;
use wgpu::CommandEncoderDescriptor;
use common::{get, get_mut_ref};
use common::singletons::settings::SETTINGS;
use common::singletons::time_package::TIME;
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

      get_mut_ref!(SETTINGS, settings); let settings = &mut settings.current_scene.parthtrace_settings;
      // settings.last_clear_frame = settings.frame; // reset to avid shizz

      let path_tracer_package = PathTracerPackage::new(render_state, *&settings);
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

      // update scene
      get_mut_ref!(SETTINGS, settings);
      let path_set = &mut settings.current_scene.parthtrace_settings;
      path_set.time = get!(TIME).start_time.elapsed().as_secs_f32();
      path_set.frame += 1;

      self.path_tracer_package.uniform.update_with_data(&render_state.queue, *&path_set);
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

   #[allow(dead_code)]
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