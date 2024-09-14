use eframe::epaint::TextureId;
use egui_wgpu::{Renderer, RenderState};
use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};

pub struct Flipper<T> {
   one: T,
   two: T,
   active: bool,
}
impl<T> Flipper<T> {
   pub fn new(one: T, two: T) -> Self {
      Self { one, two, active: false }
   }

   pub fn item_one(&self) -> &T {
      if self.active {
         &self.one
      } else {
         &self.two
      }
   }

   pub fn item_two(&self) -> &T {
      if self.active {
         &self.two
      } else {
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
   pub fn new(in_size: Extent3d, render_state: &RenderState) -> Self {
      let device = &render_state.device;
      let renderer = &mut render_state.renderer.write();

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

      let texture_id = renderer.register_native_texture(
         &device,
         &view,
         wgpu::FilterMode::Linear,
      );

      Self {
         texture,
         view,
         texture_id,
         size,
      }
   }

   pub fn update(&mut self, render_state: &RenderState) {
      if self.texture.size() != self.size {
         let size = self.size;

         self.texture.destroy();

         *self = Self::new(size, render_state);
      }
   }
}


pub fn f32_to_extent(floats: &(f32, f32)) -> Extent3d {
   Extent3d {
      width: floats.0 as u32,
      height: floats.1 as u32,
      depth_or_array_layers: 1,
   }
}

pub fn extent_to_f32(extent3d: &Extent3d) -> (f32, f32) {
   (extent3d.width as f32, extent3d.height as f32)
}