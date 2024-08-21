use eframe::wgpu;
use eframe::wgpu::{Color, CommandEncoder, Device, IndexFormat, RenderPipeline, TextureFormat, TextureView};
use eframe::wgpu::util::DeviceExt;
use crate::render_state::vertex_library::{SQUARE_INDICES, SQUARE_VERTICES};
use crate::render_state::vertex_package::{Vertex, VertexPackage};

pub struct TestRenderPipeline {
   vertex_package: VertexPackage,
   render_pipeline: RenderPipeline,
   uniform_buffer: wgpu::Buffer,
   bind_group: wgpu::BindGroup,
}

impl TestRenderPipeline {
   pub fn new(device: &Device, format: TextureFormat) -> Self {
      let vertex_package = VertexPackage::new(&device, SQUARE_VERTICES, SQUARE_INDICES);

      // Create uniform
      let uniform = 1.0;
      let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Uniform Buffer"),
         contents: bytemuck::bytes_of(&uniform),
         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

      // Create bind group layout
      let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
         entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
               ty: wgpu::BufferBindingType::Uniform,
               has_dynamic_offset: false,
               min_binding_size: None,
            },
            count: None,
         }],
         label: Some("uniform_bind_group_layout"),
      });

      // Create bind group
      let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
         layout: &bind_group_layout,
         entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
         }],
         label: Some("uniform_bind_group"),
      });

      // Render pipeline
      let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
         label: Some("Render Pipeline Layout"),
         bind_group_layouts: &[&bind_group_layout],
         push_constant_ranges: &[],
      });

      let shader = device.create_shader_module(wgpu::include_wgsl!("test_render_pipeline.wgsl"));

      let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
         label: Some("Render Pipeline"),
         layout: Some(&render_pipeline_layout),
         vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[Vertex::desc()],
         },
         fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
               format,
               blend: Some(wgpu::BlendState::REPLACE),
               write_mask: wgpu::ColorWrites::ALL,
            })],
         }),
         primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
         },
         depth_stencil: None,
         multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
         },
         multiview: None,
      });

      Self {
         vertex_package,
         render_pipeline,
         uniform_buffer,
         bind_group,
      }
   }

   pub fn update_uniform(&mut self, queue: &wgpu::Queue) {
      queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&1.0));
   }

   pub fn render_pass(&self, encoder: &mut CommandEncoder, view: &TextureView) {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
         label: Some("Render Pass"),
         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
         })],
         depth_stencil_attachment: None,
         occlusion_query_set: None,
         timestamp_writes: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_package.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.vertex_package.index_buffer.slice(..), IndexFormat::Uint16);
      render_pass.draw_indexed(0..self.vertex_package.num_indices, 0, 0..1);
   }
}