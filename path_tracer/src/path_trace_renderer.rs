use std::iter;

use eframe::CreationContext;
use eframe::emath::Vec2;
use egui::{Button, Image, Response, Sense, Ui};
use egui::load::SizedTexture;
use egui_wgpu::RenderState;
use wgpu::{CommandEncoderDescriptor, Extent3d};

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
   /// # Panics
   pub fn new(cc: &CreationContext) -> Self {
      let render_state = cc.wgpu_render_state.as_ref().expect("Couldn't unwrap render state");

      get_mut_ref!(SETTINGS, settings);

      let path_tracer_package = PathTracerPackage::new(render_state, &settings.current_scene.parthtrace_settings);
      let display_texture =
          DisplayTexture::new(render_state, path_tracer_package.storage_textures.read_layout(), &settings.image_size_settings);

      Self {
         path_tracer_package,
         display_texture,
         queue_pipeline_remake: false,
      }
   }


   pub fn update(&mut self, render_state: &RenderState) {
      get_mut_ref!(SETTINGS, settings);

      self.render_pass(render_state);

      self.display_texture.update(render_state, &settings.image_size_settings);

      if self.queue_pipeline_remake {
         self.path_tracer_package.remake_pipeline(&render_state.device);
         self.queue_pipeline_remake = false;
      }

      // update scene
      let path_set = &mut settings.current_scene.parthtrace_settings;
      path_set.time = get!(TIME).start_time.elapsed().as_secs_f32();
      path_set.frame += 1;

      self.path_tracer_package.uniform.update_with_data(&render_state.queue, path_set);

      let iss = settings.image_size_settings;
      self.path_tracer_package.storage_textures.size.width = iss.width;
      self.path_tracer_package.storage_textures.size.height = iss.height;
      self.path_tracer_package.storage_textures.update(&render_state.device);
   }


   pub fn display(&mut self, ui: &mut Ui) {
      let max = to_extent(ui.available_size());
      let mut ms = max;

      get_mut_ref!(SETTINGS, settings);
      let iss = settings.image_size_settings;

      if iss.maintain_aspect_ratio {
         let aspect = {
            let w = iss.width;
            let h = iss.height;
            h as f32 / w as f32
         };

         ms.height = (ms.width as f32 * aspect) as u32;

         if ms.height > max.height {
            let diff = ms.height as f32 - max.height as f32;

            ms.height -= diff as u32;
            ms.width -= diff as u32;
         }
      }

      self.display_texture.texture.size = ms;

      ui.horizontal(|ui| {
         // if ui.button("🔄").clicked() {
         //    self.queue_pipeline_remake = true;
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

         let mut rect = response.rect;
         rect.set_width(15.0);
         rect.set_height(15.0);

         if ui.put(rect, Button::new("🔄")).clicked() {
            self.queue_pipeline_remake = true;
         }

         self.handle_input(&response);
      });
   }

   fn handle_input(&mut self, _response: &Response) {}

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

pub fn to_extent(vec2: Vec2) -> Extent3d {
   Extent3d {
      width: vec2.x as u32,
      height: vec2.y as u32,
      depth_or_array_layers: 1,
   }
}