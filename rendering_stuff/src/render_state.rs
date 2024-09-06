use std::iter;

use egui::{Image, Key, PointerButton, Sense, Ui, Vec2};
use egui::load::SizedTexture;
use egui_wgpu::Renderer;
use wgpu::{CommandEncoderDescriptor, Device, Extent3d, Queue};

use common::{get, TIME};

use crate::path_tracer_package::PathTracePackage;
use crate::render_texture_pipeline::RenderTexturePipeline;
use crate::utility::structs::{EguiTexturePackage, RenderPack, RenderSettings};

pub struct MehRenderer {
   path_trace_package: PathTracePackage,
   display_texture: EguiTexturePackage,
   display_texture_pipeline: RenderTexturePipeline,
}

impl MehRenderer {
   pub fn new(device: &Device, queue: &Queue, renderer: &mut Renderer, render_settings: &RenderSettings) -> Self {
      let path_trace_package = PathTracePackage::new(device, queue, render_settings);

      // display texture
      let display_texture = EguiTexturePackage::new(Extent3d {
         width: 250,
         height: 250,
         depth_or_array_layers: 1,
      }, device, renderer);

      let display_texture_pipeline = RenderTexturePipeline::new(device, &path_trace_package.path_tracer_textures.pull_read());

      Self {
         path_trace_package,
         display_texture,
         display_texture_pipeline,
      }
   }

   pub fn update(&mut self, render_pack: &mut RenderPack<'_>, render_settings: &mut RenderSettings) {
      self.display_texture.update(render_pack);
      self.path_trace_package.update(render_pack, render_settings);
   }

   pub fn render_pass(&mut self, render_pack: &RenderPack<'_>) {
      let mut encoder = render_pack.device.create_command_encoder(&CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         self.path_trace_package.pass(&mut encoder);
         self.display_texture_pipeline.render_pass(&mut encoder, &self.display_texture.view, &self.path_trace_package.path_tracer_textures.pull_read());
      }

      render_pack.queue.submit(iter::once(encoder.finish()));
   }

   pub fn display(&mut self, ui: &mut Ui, render_settings: &mut RenderSettings) {
      let max = to_extent(ui.available_size());
      let mut ms = max;

      if render_settings.maintain_aspect_ratio {
         let aspect = {
            let w = render_settings.width;
            let h = render_settings.height;
            h as f32 / w as f32
         };

         ms.height = (ms.width as f32 * aspect) as u32;

         if ms.height > max.height {
            let diff = ms.height as f32 - max.height as f32;

            ms.height -= diff as u32;
            ms.width -= diff as u32;
         }
      }

      self.display_texture.size = ms;

      let response = ui.add(
         Image::from_texture(
            SizedTexture::new(
               self.display_texture.texture_id,
               Vec2::new(
                  self.display_texture.texture.size().width as f32,
                  self.display_texture.texture.size().height as f32,
               ),
            )
         ).sense(Sense::click_and_drag())
      );

      // todo shit
      let delta_time = get!(TIME).delta_time as f32;

      let cd: &mut [f32; 3] = &mut render_settings.path_tracer_uniform_settings.cam_dir;

      if response.dragged_by(PointerButton::Secondary) {
         let delta = ui.input(|i| i.pointer.delta()) * 0.005;

         cd[1] -= delta.x;
         cd[0] -= delta.y;
      }
      if response.dragged_by(PointerButton::Primary) {
         let delta = ui.input(|i| i.pointer.delta()) * 0.005;

         cd[2] += delta.x;
      }

      let speed = 5.0 * delta_time;
      ui.input(|i| {
         let pos = &mut render_settings.path_tracer_uniform_settings.cam_pos;

         // Calculate forward, right, and up vectors
         let forward = [
            cd[1].cos() * cd[0].cos(),
            cd[0].sin(),
            cd[1].sin() * cd[0].cos(),
         ];
         let right = [
            cd[1].sin(),
            0.0,
            -cd[1].cos(),
         ];
         let up = [
            -cd[0].sin() * cd[1].cos(),
            cd[0].cos(),
            -cd[0].sin() * cd[1].sin(),
         ];

         if i.key_down(Key::D) {
            pos[0] += forward[0] * speed;
            pos[1] += forward[1] * speed;
            pos[2] += forward[2] * speed;
         }
         if i.key_down(Key::A) {
            pos[0] -= forward[0] * speed;
            pos[1] -= forward[1] * speed;
            pos[2] -= forward[2] * speed;
         }
         if i.key_down(Key::W) {
            pos[0] -= right[0] * speed;
            pos[1] -= right[1] * speed;
            pos[2] -= right[2] * speed;
         }
         if i.key_down(Key::S) {
            pos[0] += right[0] * speed;
            pos[1] += right[1] * speed;
            pos[2] += right[2] * speed;
         }
         if i.key_down(Key::Q) {
            pos[0] -= up[0] * speed;
            pos[1] -= up[1] * speed;
            pos[2] -= up[2] * speed;
         }
         if i.key_down(Key::E) {
            pos[0] += up[0] * speed;
            pos[1] += up[1] * speed;
            pos[2] += up[2] * speed;
         }
      });
   }
}


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
