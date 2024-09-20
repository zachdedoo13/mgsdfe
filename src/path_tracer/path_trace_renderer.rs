use std::iter;

use eframe::CreationContext;
use eframe::emath::{Rect, Vec2};
use egui::{Button, Image, Pos2, Response, Sense, Ui};
use egui::load::SizedTexture;
use egui_wgpu::RenderState;
use wgpu::{BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, QuerySetDescriptor, QueryType};

use crate::{get, get_mut_ref};
use crate::path_tracer::display_texture_pipeline::DisplayTexture;
use crate::path_tracer::path_tracer_package::PathTracerPackage;
use crate::singletons::settings::SETTINGS;
use crate::singletons::time_package::TIME;

pub struct PathTracerRenderer {
   path_tracer_package: PathTracerPackage,
   display_texture: DisplayTexture,

   queue_pipeline_remake: bool,
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
      {
         let path_set = &mut settings.current_scene.parthtrace_settings;
         path_set.time = get!(TIME).start_time.elapsed().as_secs_f32();
         path_set.frame += 1;

         self.path_tracer_package.uniform.update_with_data(&render_state.queue, path_set);

         let iss = settings.image_size_settings;
         self.path_tracer_package.storage_textures.size.width = iss.width;
         self.path_tracer_package.storage_textures.size.height = iss.height;
         self.path_tracer_package.storage_textures.update(&render_state.device);
      }
   }

   pub fn display(&mut self, ui: &mut Ui) {
      // init
      let max = to_extent(ui.available_size());
      let mut ms = max;

      get_mut_ref!(SETTINGS, settings);
      let iss = settings.image_size_settings;

      // calc texture size
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

      // display texture
      ui.horizontal(|ui| {
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

         // refresh button
         let rect = Rect::from_center_size(response.rect.min + Pos2::new(10.0, 10.0).to_vec2(), Vec2::new(20.0, 20.0));
         if ui.put(rect, Button::new("🔄")).clicked() {
            self.queue_pipeline_remake = true;
         }

         // delegate input
         self.handle_input(ui, &response);
      });
   }

   fn handle_input(&mut self, _ui: &mut Ui, _response: &Response) {}

   fn render_pass(&mut self, render_state: &RenderState) {
      let count = 2;

      let query_set = render_state.device.create_query_set(&QuerySetDescriptor {
         label: Some("Timestamp query"),
         ty: QueryType::Timestamp,
         count,
      });

      let buffer_size = 8 * count as u64;

      let query_buffer = render_state.device.create_buffer(&BufferDescriptor {
         label: Some("query_buffer"),
         size: buffer_size,
         usage: BufferUsages::QUERY_RESOLVE |
             BufferUsages::STORAGE |
             BufferUsages::COPY_DST |
             BufferUsages::COPY_SRC,
         mapped_at_creation: false,
      });

      let cpu_buffer = render_state.device.create_buffer(&BufferDescriptor {
         label: Some("query_buffer"),
         size: buffer_size,
         usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
         mapped_at_creation: false,
      });


      let mut encoder = render_state.device.create_command_encoder(&CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      // encoder.write_timestamp(&query_set, 0);

      {
         self.path_tracer_package.render_pass(&mut encoder);
         self.display_texture.render_pass(&mut encoder, &self.path_tracer_package.storage_textures.textures.item_one().read_bind_group);
      }
      //
      // encoder.write_timestamp(&query_set, 1);
      //
      // encoder.resolve_query_set(&query_set, 0..count, &query_buffer, 0);
      //
      // encoder.copy_buffer_to_buffer(&query_buffer, 0, &cpu_buffer, 0, buffer_size);

      render_state.queue.submit(iter::once(encoder.finish()));


      // read
      // let buffer_slice = cpu_buffer.slice(..);
      // // let (sender, receiver) = flume::bounded(1);
      // // buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
      //
      // render_state.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
   }
}

pub fn to_extent(vec2: Vec2) -> Extent3d {
   Extent3d {
      width: vec2.x as u32,
      height: vec2.y as u32,
      depth_or_array_layers: 1,
   }
}