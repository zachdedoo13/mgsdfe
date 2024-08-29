use std::borrow::Cow;
use std::{iter};
use std::mem::size_of;
use bytemuck::bytes_of;
use egui::{Image, Key, PointerButton, Pos2, Sense, Ui, Vec2};
use egui::load::SizedTexture;
use egui_wgpu::Renderer;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages, Color, CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, IndexFormat, PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPipeline, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, TextureView};
use wgpu::naga::{FastHashMap, ShaderStage};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::inbuilt::vertex_library::{SQUARE_INDICES, SQUARE_VERTICES};
use crate::inbuilt::vertex_package::{Vertex, VertexPackage};
use crate::utility::structs::{DualStorageTexturePackage, EguiTexturePackage, PathTracerUniformSettings, RenderPack, RenderSettings, StorageTexturePackage};

pub struct MehRenderer {
   path_trace_package: PathTracePackage,
   display_texture: EguiTexturePackage,
   display_texture_pipeline: RenderTexturePipeline,
}

impl MehRenderer {
   pub fn new(device: &Device, renderer: &mut Renderer, render_settings: &RenderSettings) -> Self {
      let path_trace_package = PathTracePackage::new(device, render_settings);

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

      let mut cd: &mut [f32; 3] = &mut render_settings.path_tracer_uniform_settings.cam_dir;

      if response.dragged_by(PointerButton::Secondary) {
         let delta = ui.input(|i| i.pointer.delta()) * 0.005;

         cd[1] -= delta.x;
         cd[0] -= delta.y;
      }
      if response.dragged_by(PointerButton::Primary) {
         let delta = ui.input(|i| i.pointer.delta()) * 0.005;

         cd[2] += delta.x;
      }

      let speed = 0.01;
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

      println!("{cd:?}");
   }
}

fn load_shader(device: &Device, _map: String) -> ShaderModule {
   let source = include_str!("shaders/path_tracer.glsl").to_string();
   // let source = fs::read_to_string("C:/Users/zacha/RustroverProjects/mgsdfe/rendering_stuff/src/shaders/path_tracer.glsl").unwrap(); // for testing only

   let shader_mod = ShaderModuleDescriptor {
      label: None,
      source: ShaderSource::Glsl {
         shader: Cow::Owned(source),
         stage: ShaderStage::Compute,
         defines: FastHashMap::default(), // Adjust as needed for your shader
      },
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
            &texture_package.read_bind_group_layout,
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
               },
            })
         ],
         depth_stencil_attachment: None,
         occlusion_query_set: None,
         timestamp_writes: None,
      });

      render_pass.set_pipeline(&self.pipeline);

      render_pass.set_bind_group(0, &texture_package.read_bind_group, &[]);

      render_pass.set_vertex_buffer(0, self.vertex_package.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.vertex_package.index_buffer.slice(..), IndexFormat::Uint16);

      render_pass.draw_indexed(0..self.vertex_package.num_indices, 0, 0..1);
   }
}


pub struct PathTracerUniform {
   bind_group: BindGroup,
   layout: BindGroupLayout,
   buffer: Buffer,
}

impl PathTracerUniform {
   pub fn new(device: &Device, data: &PathTracerUniformSettings) -> Self {
      let buffer = device.create_buffer_init(&BufferInitDescriptor {
         label: Some("PathTracerUniform"),
         contents: bytes_of(data),
         usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      });

      let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
         label: Some("UniformPackageSingles"),
         entries: &[
            wgpu::BindGroupLayoutEntry {
               binding: 0,
               visibility: ShaderStages::COMPUTE,
               ty: wgpu::BindingType::Buffer {
                  ty: wgpu::BufferBindingType::Uniform,
                  has_dynamic_offset: false,
                  min_binding_size: wgpu::BufferSize::new(size_of::<PathTracerUniformSettings>() as u64),
               },
               count: None,
            },
         ],
      });

      let bind_group = device.create_bind_group(&BindGroupDescriptor {
         label: None,
         layout: &layout,
         entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
         }],
      });

      Self {
         buffer,
         bind_group,
         layout,
      }
   }

   pub fn update_with_data(&self, queue: &Queue, data: PathTracerUniformSettings) {
      queue.write_buffer(
         &self.buffer,
         0,
         bytes_of(&data),
      );
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


pub struct PathTracePackage {
   path_tracer_pipeline_layout: PipelineLayout,
   path_tracer_pipeline: ComputePipeline,
   path_tracer_textures: DualStorageTexturePackage,
   path_tracer_uniform: PathTracerUniform,
}

impl PathTracePackage {
   pub fn new(device: &Device, render_settings: &RenderSettings) -> Self {
      let path_tracer_uniform = PathTracerUniform::new(device, &RenderSettings::default().path_tracer_uniform_settings);

      let one = StorageTexturePackage::new(device, (render_settings.width as f32, render_settings.height as f32));
      let two = StorageTexturePackage::new(device, (render_settings.width as f32, render_settings.height as f32));
      let path_tracer_textures = DualStorageTexturePackage::new(one, two);
      let refs = path_tracer_textures.pull_both();

      let path_tracer_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
         label: Some("path_tracer_pipeline_layout"),
         bind_group_layouts: &[
            &refs.read.read_bind_group_layout,
            &refs.write.write_bind_group_layout,
            &path_tracer_uniform.layout,
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

      Self {
         path_tracer_pipeline_layout,
         path_tracer_pipeline,
         path_tracer_textures,
         path_tracer_uniform,
      }
   }

   pub fn update(&mut self, render_pack: &mut RenderPack<'_>, render_settings: &mut RenderSettings) {
      let check = (render_settings.width, render_settings.height);

      self.path_tracer_uniform.update_with_data(&render_pack.queue, render_settings.path_tracer_uniform_settings);
      self.path_tracer_textures.update(&render_pack.device, check);

      if render_settings.remake_pipeline {
         self.remake_pipeline(&render_pack.device);
         render_settings.remake_pipeline = false;
      }
   }

   pub fn pass(&mut self, encoder: &mut CommandEncoder) {
      let refs = self.path_tracer_textures.pull_both();

      {
         let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("path_tracer_pipeline"),
            timestamp_writes: None,
         });

         compute_pass.set_pipeline(&self.path_tracer_pipeline);

         // bind groups
         compute_pass.set_bind_group(0, &refs.read.read_bind_group, &[]);
         compute_pass.set_bind_group(1, &refs.write.write_bind_group, &[]);
         compute_pass.set_bind_group(2, &self.path_tracer_uniform.bind_group, &[]);

         let size = refs.read.size;
         let wg = 16;
         compute_pass.dispatch_workgroups(
            (size.width as f32 / wg as f32).ceil() as u32,
            (size.height as f32 / wg as f32).ceil() as u32,
            1,
         );
      } // `compute_pass` is dropped here

      // Perform the flip after the immutable borrows are done
      self.path_tracer_textures.flip();
   }

   fn remake_pipeline(&mut self, device: &Device) {
      let shader_module = load_shader(device, String::new());

      self.path_tracer_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("path_tracer_pipeline"),
            layout: Some(&self.path_tracer_pipeline_layout),
            module: &shader_module,
            entry_point: "main",
            compilation_options: Default::default(),
         });

   }
}