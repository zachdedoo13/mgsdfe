use std::borrow::Cow;
use std::{fs, iter};
use eframe::egui_wgpu::Renderer;
use egui::load::SizedTexture;
use egui::{Image, Ui, Vec2};
use wgpu::{Color, CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, IndexFormat, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline, ShaderModule, ShaderModuleDescriptor, ShaderSource, TextureFormat, TextureView};
use wgpu::naga::{FastHashMap, ShaderStage};
use crate::app::RENDER_SETTINGS;
use crate::get;
use crate::render_state::structs::{EguiTexturePackage};
use crate::utility::functions::to_extent;
use crate::render_state::structs::RenderPack;
use crate::render_state::vertex_library::{SQUARE_INDICES, SQUARE_VERTICES};
use crate::render_state::vertex_package::{Vertex, VertexPackage};
use crate::utility::structs::StorageTexturePackage;



// is a static in app.rs
pub struct RenderSettings {
   pub width: u32,
   pub height: u32,
}
impl RenderSettings {
   pub fn new() -> Self {
      Self {
         width: 250,
         height: 250,
      }
   }
}



pub struct MehRenderer {
   path_tracer_pipeline_layout: PipelineLayout,
   path_tracer_pipeline: ComputePipeline,
   path_tracer_texture: StorageTexturePackage,
   //
   // post_process_pipeline: RenderPipeline,
   // post_process_texture: StorageTexturePackage,

   display_texture: EguiTexturePackage,
   display_texture_pipeline: RenderTexturePipeline,
}
impl MehRenderer {
   pub fn new(device: &Device, renderer: &mut Renderer) -> Self {

      let render_settings = &get!(RENDER_SETTINGS);

      // Path tracer
      let path_tracer_texture = StorageTexturePackage::new(device, (render_settings.width as f32, render_settings.height as f32));

      let path_tracer_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
         label: Some("path_tracer_pipeline_layout"),
         bind_group_layouts: &[
            &path_tracer_texture.bind_group_layout,
         ],
         push_constant_ranges: &[],
      });

      let shader_module = load_shader(device, String::new());

      let path_tracer_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
         label: Some("path_tracer_pipeline"),
         layout: Some(&path_tracer_pipeline_layout),
         module: &shader_module,
         entry_point: "main",
         compilation_options: Default::default(),
      });

      let display_texture = EguiTexturePackage::new(Extent3d {
         width: 250,
         height: 250,
         depth_or_array_layers: 1,
      }, device, renderer);

      let display_texture_pipeline = RenderTexturePipeline::new(device, &path_tracer_texture);

      Self {
         path_tracer_pipeline_layout,
         path_tracer_pipeline,
         path_tracer_texture,

         display_texture,
         display_texture_pipeline,
      }
   }

   pub fn update(&mut self, render_pack: &mut RenderPack<'_>) {
      self.display_texture.update(render_pack);

      let check = {
         let rs = &get!(RENDER_SETTINGS);
         (rs.width, rs.height)
      };

      self.path_tracer_texture.update(&render_pack.device, check);
   }

   pub fn render_pass(&self, render_pack: &RenderPack<'_>) {
      let view = &self.display_texture.view;
      let mut encoder = render_pack.device.create_command_encoder(&CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         self.compute_pass(&mut encoder);
         self.display_texture_pipeline.render_pass(&mut encoder, &self.display_texture.view, &self.path_tracer_texture);
      }

      render_pack.queue.submit(iter::once(encoder.finish()));
   }

   pub fn display(&mut self, ui: &mut Ui) {
      let ms = to_extent(ui.available_size());
      self.display_texture.size = ms;

      ui.add(
         Image::from_texture(
            SizedTexture::new(
               self.display_texture.texture_id,
               Vec2::new(
                  self.display_texture.texture.size().width as f32,
                  self.display_texture.texture.size().height as f32
               )
            )
         )
      );
   }


   fn compute_pass(&self, encoder: &mut CommandEncoder) {
      let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
         label: Some("path_tracer_pipeline"),
         timestamp_writes: None
      });

      compute_pass.set_pipeline(&self.path_tracer_pipeline);

      // bind groups
      compute_pass.set_bind_group(0, &self.path_tracer_texture.bind_group, &[]);

      //

      let wg = 16;
      compute_pass.dispatch_workgroups(
         (self.path_tracer_texture.size.width as f32 / wg as f32).ceil() as u32,
         (self.path_tracer_texture.size.height as f32 / wg as f32).ceil() as u32,
         1);
   }
}


fn load_shader(device: &Device, map: String) -> ShaderModule {
   // let source = fs::read_to_string("src/render_state/test/test_compute.glsl").unwrap();
   let source = include_str!("test/test_compute.glsl").to_string();

   let shader_mod = ShaderModuleDescriptor {
      label: None,
      source: ShaderSource::Glsl {
         shader: Cow::Owned(source),
         stage: ShaderStage::Compute,
         defines: FastHashMap::default(), // Adjust as needed for your shader
      }
   };

   device.create_shader_module(shader_mod)
}





pub struct RenderTexturePipeline {
   vertex_package: VertexPackage,
   pub pipeline: RenderPipeline,
}
impl RenderTexturePipeline {
   pub fn new(device: &Device, texture_package: &StorageTexturePackage) -> Self {
      let vertex_package = VertexPackage::new(&device, SQUARE_VERTICES, SQUARE_INDICES);

      let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
         label: Some("Render Pipeline Layout"),
         bind_group_layouts: &[
            &texture_package.bind_group_layout,
         ],
         push_constant_ranges: &[],
      });

      let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render_texture_shader.wgsl"));


      let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
         label: Some("Render Pipeline"),
         layout: Some(&render_pipeline_layout),

         vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main", // 1.
            compilation_options: Default::default(),
            buffers: &[
               Vertex::desc(),
            ], // 2.
         },

         fragment: Some(wgpu::FragmentState { // 3.
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState { // 4.
               format: TextureFormat::Rgba8Unorm,
               blend: Some(wgpu::BlendState::REPLACE),
               write_mask: wgpu::ColorWrites::ALL,
            })],
         }),

         primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
         },

         depth_stencil: None, // 1.
         multisample: wgpu::MultisampleState {
            count: 1, // 2.
            mask: !0, // 3. returns a bit array of all ones to select all possible masks 0x1111...
            alpha_to_coverage_enabled: false, // 4.
         },

         multiview: None, // 5.
      });

      Self {
         vertex_package,
         pipeline,
      }
   }

   pub fn render_pass(&self, encoder: &mut CommandEncoder, view: &TextureView, texture_package: &StorageTexturePackage) {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
         label: Some("Render Pass"),
         color_attachments: &[
            // This is what @location(0) in the fragment shader targets
            Some(wgpu::RenderPassColorAttachment {
               view: &view,
               resolve_target: None,
               ops: wgpu::Operations {
                  load: wgpu::LoadOp::Clear(Color {
                     r: 0.0,
                     g: 0.0,
                     b: 0.0,
                     a: 1.0,
                  }),
                  store: wgpu::StoreOp::Store,
               }
            })
         ],
         depth_stencil_attachment: None,
         occlusion_query_set: None,
         timestamp_writes: None,
      });

      render_pass.set_pipeline(&self.pipeline);

      render_pass.set_bind_group(0, &texture_package.bind_group, &[]);

      render_pass.set_vertex_buffer(0, self.vertex_package.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.vertex_package.index_buffer.slice(..), IndexFormat::Uint16);

      render_pass.draw_indexed(0..self.vertex_package.num_indices, 0, 0..1);

   }
}





