use eframe::wgpu;
use eframe::wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Device, Extent3d, ShaderStages, StorageTextureAccess, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};

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


pub struct Flipper<T> {
   one: T,
   two: T,
   active: bool,
}
impl<T> Flipper<T> {
   pub fn new(one: T, two: T) -> Self {
      Self {one, two, active: false}
   }

   pub fn item_one(&mut self) -> &mut T {
      if self.active {
         &mut self.one
      }
      else {
         &mut self.two
      }
   }

   pub fn item_two(&mut self) -> &mut T {
      if self.active {
         &mut self.two
      }
      else {
         &mut self.one
      }
   }

   pub fn flip(&mut self) {
      self.active = !self.active;
   }
}