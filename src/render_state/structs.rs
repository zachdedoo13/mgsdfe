use eframe::{egui_wgpu, wgpu};
use eframe::egui::TextureId;
use eframe::wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};

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

   pub fn update(&mut self, device: &Device, renderer: &mut egui_wgpu::Renderer) {
      if self.texture.size() != self.size {
         let size = self.size;

         self.texture.destroy();

         *self = Self::new(size, device, renderer);
      }
   }
}