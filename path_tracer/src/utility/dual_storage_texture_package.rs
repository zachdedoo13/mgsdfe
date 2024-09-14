use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Device, Extent3d, ShaderStages, StorageTextureAccess, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};

use crate::utility::helper_structs::{extent_to_f32, f32_to_extent, Flipper};

pub struct DualStorageTexturePackage {
   pub size: Extent3d,
   pub textures: Flipper<StorageTexturePackage>,
}
impl DualStorageTexturePackage {
   pub fn new(device: &Device) -> Self {
      let size = Extent3d {
         width: 250,
         height: 250,
         depth_or_array_layers: 1,
      };

      let tex_1 = StorageTexturePackage::new(device, extent_to_f32(&size));
      let tex_2 = StorageTexturePackage::new(device, extent_to_f32(&size));

      let textures = Flipper::new(tex_1, tex_2);

      Self {
         size,
         textures,
      }
   }

   pub fn read_layout(&self) -> &BindGroupLayout {
      &self.textures.item_one().read_bind_group_layout
   }

   pub fn write_layout(&self) -> &BindGroupLayout {
      &self.textures.item_one().write_bind_group_layout
   }
}


pub struct StorageTexturePackage {
   pub texture: Texture,
   pub view: TextureView,

   pub read_bind_group_layout: BindGroupLayout,
   pub read_bind_group: BindGroup,

   pub write_bind_group_layout: BindGroupLayout,
   pub write_bind_group: BindGroup,
}
impl StorageTexturePackage {
   pub fn new(device: &Device, in_size: (f32, f32)) -> Self {
      let size = f32_to_extent(&in_size);

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
         self.remake(device, (size_check.0 as f32, size_check.1 as f32));
      }
   }

   pub fn remake(&mut self, device: &Device, size: (f32, f32)) {
      *self = Self::new(device, size);
   }
}