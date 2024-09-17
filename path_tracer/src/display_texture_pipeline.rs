use egui_wgpu::RenderState;
use wgpu::{BindGroup, BindGroupLayout, Color, CommandEncoder, IndexFormat, PipelineLayoutDescriptor, RenderPipeline, TextureFormat};

use crate::utility::helper_structs::{EguiTexturePackage, f32_to_extent};
use crate::utility::vertex_library::{SQUARE_INDICES, SQUARE_VERTICES};
use crate::utility::vertex_package::{Vertex, VertexPackage};

pub struct DisplayTexture {
   vertex_package: VertexPackage,
   pub pipeline: RenderPipeline,
   pub texture: EguiTexturePackage,
}

impl DisplayTexture {
   pub fn new(render_state: &RenderState, read_bindgroup_layout: &BindGroupLayout) -> Self {
      let device = &render_state.device;
      let vertex_package = VertexPackage::new(&device, SQUARE_VERTICES, SQUARE_INDICES);

      let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
         label: Some("Render Pipeline Layout"),
         bind_group_layouts: &[
            read_bindgroup_layout,
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

      let texture = EguiTexturePackage::new(f32_to_extent(&(250.0, 250.0)), render_state);

      Self {
         vertex_package,
         pipeline,
         texture,
      }
   }

   pub fn update(&mut self, render_state: &RenderState) {
      self.texture.update(render_state);
   }

   pub fn render_pass(&self, encoder: &mut CommandEncoder, read_bindgroup: &BindGroup) {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
         label: Some("Render Pass"),
         color_attachments: &[
            // This is what @location(0) in the fragment shader targets
            Some(wgpu::RenderPassColorAttachment {
               view: &self.texture.view,
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

      render_pass.set_bind_group(0, read_bindgroup, &[]);

      render_pass.set_vertex_buffer(0, self.vertex_package.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.vertex_package.index_buffer.slice(..), IndexFormat::Uint16);

      render_pass.draw_indexed(0..self.vertex_package.num_indices, 0, 0..1);
   }
}