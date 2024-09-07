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

      if response.dragged_by(PointerButton::Primary) {
         let delta = ui.input(|i| i.pointer.delta()) * 0.25;

         cd[1] += delta.x;
         cd[0] = (cd[0] + delta.y);
             // .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
      }

      let speed = 5.0 * delta_time;
      ui.input(|i| {
         let pos = &mut render_settings.path_tracer_uniform_settings.cam_pos;

         // Calculate forward, right, and up vectors
         let forward = rotate_ray_direction([0.0, 0.0, -1.0], cd.clone());
         let right = rotate_ray_direction([1.0, 0.0, 0.0], cd.clone());

         if i.key_down(Key::W) {
            pos[0] -= forward[0] * speed;
            // pos[1] -= forward[1] * speed;
            pos[2] -= forward[2] * speed;
         }

         if i.key_down(Key::S) {
            pos[0] += forward[0] * speed;
            // pos[1] += forward[1] * speed;
            pos[2] += forward[2] * speed;
         }

         if i.key_down(Key::D) {
            pos[0] += right[0] * speed;
            // pos[1] += right[1] * speed;
            pos[2] += right[2] * speed;
         }

         if i.key_down(Key::A) {
            pos[0] -= right[0] * speed;
            // pos[1] -= right[1] * speed;
            pos[2] -= right[2] * speed;
         }

         if i.key_down(Key::Q) {
            pos[1] += speed;
         }

         if i.key_down(Key::E) {
            pos[1] -= speed;
         }

      });
   }
}

fn rotate_ray_direction(direction: [f32; 3], rotation: [f32; 3]) -> [f32; 3] {
   let rad = [
      rotation[0].to_radians(),
      rotation[1].to_radians(),
      rotation[2].to_radians(),
   ];

   let half_angle_x = rad[0] * 0.5;
   let s_x = half_angle_x.sin();
   let w_x = half_angle_x.cos();
   let xyz_x = [1.0 * s_x, 0.0, 0.0];

   let half_angle_y = rad[1] * 0.5;
   let s_y = half_angle_y.sin();
   let w_y = half_angle_y.cos();
   let xyz_y = [0.0, 1.0 * s_y, 0.0];

   let half_angle_z = rad[2] * 0.5;
   let s_z = half_angle_z.sin();
   let w_z = half_angle_z.cos();
   let xyz_z = [0.0, 0.0, 1.0 * s_z];

   let w_yx = w_y * w_x - (xyz_y[0] * xyz_x[0] + xyz_y[1] * xyz_x[1] + xyz_y[2] * xyz_x[2]);
   let xyz_yx = [
      w_y * xyz_x[0] + w_x * xyz_y[0] + (xyz_y[1] * xyz_x[2] - xyz_y[2] * xyz_x[1]),
      w_y * xyz_x[1] + w_x * xyz_y[1] + (xyz_y[2] * xyz_x[0] - xyz_y[0] * xyz_x[2]),
      w_y * xyz_x[2] + w_x * xyz_y[2] + (xyz_y[0] * xyz_x[1] - xyz_y[1] * xyz_x[0]),
   ];

   let w = w_z * w_yx - (xyz_z[0] * xyz_yx[0] + xyz_z[1] * xyz_yx[1] + xyz_z[2] * xyz_yx[2]);
   let xyz = [
      w_z * xyz_yx[0] + w_yx * xyz_z[0] + (xyz_z[1] * xyz_yx[2] - xyz_z[2] * xyz_yx[1]),
      w_z * xyz_yx[1] + w_yx * xyz_z[1] + (xyz_z[2] * xyz_yx[0] - xyz_z[0] * xyz_yx[2]),
      w_z * xyz_yx[2] + w_yx * xyz_z[2] + (xyz_z[0] * xyz_yx[1] - xyz_z[1] * xyz_yx[0]),
   ];

   let t = [
      2.0 * (xyz[1] * direction[2] - xyz[2] * direction[1]),
      2.0 * (xyz[2] * direction[0] - xyz[0] * direction[2]),
      2.0 * (xyz[0] * direction[1] - xyz[1] * direction[0]),
   ];

   [
      direction[0] + w * t[0] + (xyz[1] * t[2] - xyz[2] * t[1]),
      direction[1] + w * t[1] + (xyz[2] * t[0] - xyz[0] * t[2]),
      direction[2] + w * t[2] + (xyz[0] * t[1] - xyz[1] * t[0]),
   ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
   let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
   if mag == 0.0 {
      return [0.0, 0.0, 0.0];
   }
   [v[0] / mag, v[1] / mag, v[2] / mag]
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
