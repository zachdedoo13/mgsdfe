use bytemuck::{Pod, Zeroable};
use eframe::{egui_wgpu, wgpu};
use eframe::egui::TextureId;
use eframe::egui_wgpu::Renderer;
use eframe::wgpu::{Device, Extent3d, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};
use serde::{Deserialize, Serialize};

pub struct EguiTexturePackage {
   pub texture: Texture,
   pub view: TextureView,
   pub texture_id: TextureId,
   pub size: Extent3d,
}
impl EguiTexturePackage {
   pub fn new(in_size: Extent3d, device: &Device, renderer: &mut egui_wgpu::Renderer) -> Self {
      let size = Extent3d {
         width: if in_size.width > 0 { in_size.width } else { 1 },
         height: if in_size.height > 0 { in_size.height } else { 1 },
         depth_or_array_layers: 1,
      };

      let texture = device.create_texture(&TextureDescriptor {
         label: Some("Egui Texture"),
         size: size.clone(),
         mip_level_count: 1,
         sample_count: 1,
         dimension: TextureDimension::D2,
         format: TextureFormat::Rgba8Unorm,
         usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
         view_formats: &[],
      });

      let view = texture.create_view(&TextureViewDescriptor::default());

      let texture_id =  renderer.register_native_texture(
         &device,
         &view,
         wgpu::FilterMode::Linear
      );

      Self {
         texture,
         view,
         texture_id,
         size,
      }
   }

   pub fn update(&mut self, render_pack: &mut RenderPack<'_>) {
      if self.texture.size() != self.size {
         let size = self.size;

         self.texture.destroy();

         *self = Self::new(size, render_pack.device, render_pack.renderer);
      }
   }
}


pub struct RenderPack<'a> {
   pub device: &'a Device,
   pub queue: &'a Queue,
   pub renderer: &'a mut Renderer,
}



// is a static in app.rs
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct RenderSettings {
   pub width: u32,
   pub height: u32,

   pub maintain_aspect_ratio: bool,

   pub path_tracer_uniform_settings: PathTracerUniformSettings,
}

impl Default for RenderSettings {
   fn default() -> Self {
      Self {
         width: 250,
         height: 250,

         maintain_aspect_ratio: true,

         path_tracer_uniform_settings: PathTracerUniformSettings::default(),
      }
   }
}

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable, Serialize, Deserialize)]
pub struct PathTracerUniformSettings {
   pub time: f32,
   pub frame: i32,
   pub last_clear_frame: i32,

   pub samples_per_frame: i32,

   pub steps_per_ray: i32,
   pub bounces: i32,
   pub fov: f32,

   pub start_eps: f32,
   pub max_dist: f32,
   pub relaxation: f32,
   pub step_scale_factor: f32,
   pub eps_scale: f32,

   pub cam_pos: [f32; 3],
   // pub cam_dir: [f32; 3],
}

impl Default for PathTracerUniformSettings {
   fn default() -> Self {
      Self {
         time: 0.0,
         frame: 0,
         last_clear_frame: 0,

         samples_per_frame: 1,

         steps_per_ray: 60,
         bounces: 8,
         fov: 90.0,

         start_eps: 0.001,
         max_dist: 16.0,
         relaxation: 1.021,
         step_scale_factor: 1.137,
         eps_scale: 1.136,


         cam_pos: [0.0, 0.0, 0.0],
         // cam_dir: [0.0, 0.0, 0.0],
         //
         // buffer: [0.0; 2],
      }
   }
}