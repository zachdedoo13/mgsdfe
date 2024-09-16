use std::borrow::Cow;
use std::mem::size_of;
use std::panic::AssertUnwindSafe;

use bytemuck::{bytes_of, Pod, Zeroable};
use egui_wgpu::RenderState;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayout, PipelineLayoutDescriptor, Queue, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};
use wgpu::naga::{FastHashMap, ShaderStage};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::utility::dual_storage_texture_package::DualStorageTexturePackage;

pub struct PathTracerPackage {
   pub pipeline_layout: PipelineLayout,
   pub compute_pipeline: ComputePipeline,
   pub storage_textures: DualStorageTexturePackage,
   pub uniform: PathTracerUniform,
}
impl PathTracerPackage {
   pub fn new(render_state: &RenderState) -> Self {
      let device = &render_state.device;

      let storage_textures = DualStorageTexturePackage::new(device);

      let shader_module = load_shader(device, String::new()).expect("Must start with a valid shader module"); // todo placeholder

      let uniform = PathTracerUniform::new(device, PathTracerSettings::default()); // todo also placeholder

      let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
         label: Some("PathTracerPackage pipeline_layout"),
         bind_group_layouts: &[
            storage_textures.read_layout(),
            storage_textures.write_layout(),
            &uniform.layout,
         ],
         push_constant_ranges: &[],
      });

      let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
         label: Some("PathTracerPackage compute_pipeline"),
         layout: Some(&pipeline_layout),
         module: &shader_module,
         entry_point: "main",
         compilation_options: Default::default(),
      });

      Self {
         pipeline_layout,
         compute_pipeline,
         storage_textures,
         uniform,
      }
   }

   pub fn render_pass(&mut self, encoder: &mut CommandEncoder) {
      {
         let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("path_tracer_pipeline"),
            timestamp_writes: None,
         });

         compute_pass.set_pipeline(&self.compute_pipeline);

         // bind groups
         compute_pass.set_bind_group(0, &self.storage_textures.textures.item_one().read_bind_group, &[]);
         compute_pass.set_bind_group(1, &self.storage_textures.textures.item_two().write_bind_group, &[]);
         compute_pass.set_bind_group(2, &self.uniform.bind_group, &[]);

         let size = self.storage_textures.size;
         let wg = 16;
         compute_pass.dispatch_workgroups(
            (size.width as f32 / wg as f32).ceil() as u32,
            (size.height as f32 / wg as f32).ceil() as u32,
            1,
         );
      }

      // Perform the flip after the immutable borrows are done
      self.storage_textures.textures.flip();
   }
}

impl PathTracerPackage {
   pub fn remake_pipeline(&mut self, device: &Device) {
      let shader_module = load_shader(device, String::new());

      if let Ok(sm) = shader_module {
         self.compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("path_tracer_pipeline"),
            layout: Some(&self.pipeline_layout),
            module: &sm,
            entry_point: "main",
            compilation_options: Default::default(),
         });
      }
   }
}


fn load_shader(device: &Device, _map: String) -> std::thread::Result<ShaderModule> {
   // let mapped_code = include_str!("shaders/testing.glsl").to_string(); // todo placeholder
   let mapped_code = std::fs::read_to_string("path_tracer/src/shaders/testing.glsl").unwrap();

   let source = mapped_code;

   let shader_mod = ShaderModuleDescriptor {
      label: None,
      source: ShaderSource::Glsl {
         shader: Cow::Owned(source),
         stage: ShaderStage::Compute,
         defines: FastHashMap::default(), // Adjust as needed for your shader
      },
   };

   let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
      let prev_hook = std::panic::take_hook();
      std::panic::set_hook(Box::new(|panic_info| {
         if let Some(_) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Panic occurred");
         } else {
            eprintln!("Panic occurred");
         }
         eprintln!("Occurred during the shader compilation");
      }));
      let result = device.create_shader_module(shader_mod);

      std::panic::set_hook(prev_hook);

      result
   }));

   if let Ok(_) = out {
      println!("Compiled")
   }

   out
}


#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct PathTracerSettings {
   pub time: f32,
   pub frame: i32,

   pub last_clear_frame: i32,
   pub samples_per_frame: i32,
   pub steps_per_ray: i32,

   pub bounces: i32,
   pub fov: f32,

   pub cam_pos: [f32; 3],
   pub cam_dir: [f32; 3],
}
impl Default for PathTracerSettings {
   fn default() -> Self {
      Self {
         time: 0.0,
         frame: 0,

         last_clear_frame: 0,
         samples_per_frame: 1,
         steps_per_ray: 60,

         bounces: 8,
         fov: 90.0,

         cam_pos: [0.0, 0.0, 0.0],
         cam_dir: [1.0, 2.0, 3.0],
      }
   }
}


pub struct PathTracerUniform {
   bind_group: BindGroup,
   layout: BindGroupLayout,
   buffer: Buffer,
   data: PathTracerSettings,
}
impl PathTracerUniform {
   pub fn new(device: &Device, data: PathTracerSettings) -> Self {
      let buffer = device.create_buffer_init(&BufferInitDescriptor {
         label: Some("PathTracerUniform"),
         contents: bytes_of(&data),
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
                  min_binding_size: wgpu::BufferSize::new(size_of::<PathTracerSettings>() as u64),
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
         data,
      }
   }

   pub fn update_with_data(&self, queue: &Queue) {
      queue.write_buffer(
         &self.buffer,
         0,
         bytes_of(&self.data),
      );
   }
}