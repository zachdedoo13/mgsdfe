use bytemuck::{Pod, Zeroable};
use egui::TextureId;
use egui_wgpu::Renderer;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Device, Extent3d, Queue, ShaderStages, StorageTextureAccess, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};
use serde::{Deserialize, Serialize};

pub struct StorageTexturePackage {
   pub size: Extent3d,
   pub texture: Texture,
   pub view: TextureView,

   pub read_bind_group_layout: BindGroupLayout,
   pub read_bind_group: BindGroup,

   pub write_bind_group_layout: BindGroupLayout,
   pub write_bind_group: BindGroup,
}
impl StorageTexturePackage {
   pub fn new(device: &Device, size: (f32, f32)) -> Self {
      let size = Extent3d {
         width: size.0 as u32,
         height: size.1 as u32,
         // width: 128,
         // height: 128,
         depth_or_array_layers: 1,
      };

      let texture_desc = TextureDescriptor {
         label: Some("test"),
         size,
         mip_level_count: 1,
         sample_count: 1,
         dimension: TextureDimension::D2,
         format: TextureFormat::Rgba32Float,
         usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
         view_formats: &[],
      };

      let texture = device.create_texture(&texture_desc);
      let view = texture.create_view(&TextureViewDescriptor::default());


      let read_bind_group_layout =
          device.create_bind_group_layout(&BindGroupLayoutDescriptor {
             entries: &[
                wgpu::BindGroupLayoutEntry {
                   binding: 0,
                   visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                   ty: wgpu::BindingType::StorageTexture {
                      access: StorageTextureAccess::ReadOnly,
                      format: TextureFormat::Rgba32Float,
                      view_dimension: TextureViewDimension::D2,
                   },
                   count: None,
                },
             ],
             label: Some("texture_bind_group_layout Read Only"),
          });

      let read_bind_group = device.create_bind_group(&BindGroupDescriptor {
         layout: &read_bind_group_layout,
         entries: &[
            BindGroupEntry {
               binding: 0,
               resource: wgpu::BindingResource::TextureView(&view),
            },
         ],
         label: Some("diffuse_bind_group  Read Only"),
      });


      let write_bind_group_layout =
          device.create_bind_group_layout(&BindGroupLayoutDescriptor {
             entries: &[
                wgpu::BindGroupLayoutEntry {
                   binding: 0,
                   visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                   ty: wgpu::BindingType::StorageTexture {
                      access: StorageTextureAccess::WriteOnly,
                      format: TextureFormat::Rgba32Float,
                      view_dimension: TextureViewDimension::D2,
                   },
                   count: None,
                },
             ],
             label: Some("texture_bind_group_layout Write only"),
          });

      let write_bind_group = device.create_bind_group(&BindGroupDescriptor {
         layout: &write_bind_group_layout,
         entries: &[
            BindGroupEntry {
               binding: 0,
               resource: wgpu::BindingResource::TextureView(&view),
            },
         ],
         label: Some("diffuse_bind_group  Read Only"),
      });


      Self {
         size,
         texture,
         view,

         read_bind_group_layout,
         read_bind_group,

         write_bind_group_layout,
         write_bind_group,
      }
   }

   pub fn update(&mut self, device: &Device, size_check: (u32, u32)) {
      if (self.texture.width() != size_check.0) | (self.texture.height() != size_check.1) {
         self.texture.destroy();
         *self = Self::new(device, (size_check.0 as f32, size_check.1 as f32));
      }
   }

   pub fn remake(&mut self, device: &Device, size: (f32, f32)) {
      *self = Self::new(device, size);
   }
}


pub struct DualStorageTexturePackage {
   textures: Flipper<StorageTexturePackage>,
}
impl DualStorageTexturePackage {
   pub fn new(one: StorageTexturePackage, two: StorageTexturePackage) -> Self {
      Self {
         textures: Flipper::new(one, two),
      }
   }

   pub fn pull_both(&self) -> DualOutput<'_> {
      let read = self.textures.item_one();
      let write = self.textures.item_two();

      DualOutput {
         read,
         write,
      }
   }

   pub fn pull_read(&self) -> &StorageTexturePackage {
      self.textures.item_one()
   }

   pub fn pull_write(&self) -> &StorageTexturePackage {
      self.textures.item_two()
   }


   pub fn update(&mut self, device: &Device, size_check: (u32, u32)) {
      self.textures.one.update(device, size_check);
      self.textures.two.update(device, size_check);
   }

   pub fn flip(&mut self) {
      self.textures.flip();
   }
}

pub struct DualOutput<'a> {
   pub read: &'a StorageTexturePackage,
   pub write: &'a StorageTexturePackage,
}



pub struct Flipper<T> {
   one: T,
   two: T,
   active: bool,
}
impl<T> Flipper<T> {
   pub fn new(one: T, two: T) -> Self {
      Self {one, two, active: false}
   }

   pub fn item_one(&self) -> &T {
      if self.active {
         &self.one
      }
      else {
         &self.two
      }
   }

   pub fn item_two(&self) -> &T {
      if self.active {
         &self.two
      }
      else {
         &self.one
      }
   }

   pub fn flip(&mut self) {
      self.active = !self.active;
   }
}



pub struct EguiTexturePackage {
   pub texture: Texture,
   pub view: TextureView,
   pub texture_id: TextureId,
   pub size: Extent3d,
}
impl EguiTexturePackage {
   pub fn new(in_size: Extent3d, device: &Device, renderer: &mut Renderer) -> Self {
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