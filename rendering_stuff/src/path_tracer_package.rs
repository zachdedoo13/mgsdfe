use std::borrow::Cow;
use std::mem::size_of;
use std::panic::AssertUnwindSafe;
use bytemuck::bytes_of;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayout, PipelineLayoutDescriptor, Queue, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};
use wgpu::naga::{FastHashMap, ShaderStage};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::utility::structs::{DualStorageTexturePackage, PathTracerUniformSettings, RenderPack, RenderSettings, SampledTexturePackage, StorageTexturePackage};




pub struct PathTracePackage {
   pub path_tracer_pipeline_layout: PipelineLayout,
   pub path_tracer_pipeline: ComputePipeline,
   pub path_tracer_textures: DualStorageTexturePackage,
   pub path_tracer_uniform: PathTracerUniform,

   pub test_tex: SampledTexturePackage,
}

impl PathTracePackage {
   pub fn new(device: &Device, queue: &Queue, render_settings: &RenderSettings) -> Self {
      let path_tracer_uniform = PathTracerUniform::new(device, &RenderSettings::default().path_tracer_uniform_settings);

      let london: &'static [u8] = include_bytes!("shaders/assets/london.jpg");
      let test_tex = SampledTexturePackage::new(device, queue, london);

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
            &test_tex.bind_group_layout,
         ],
         push_constant_ranges: &[],
      });

      let shader_module = load_shader(device, String::new()).unwrap();

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
         test_tex,
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

         compute_pass.set_bind_group(3, &self.test_tex.bind_group, &[]);

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

      if let Ok(sm) = shader_module {
         self.path_tracer_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("path_tracer_pipeline"),
            layout: Some(&self.path_tracer_pipeline_layout),
            module: &sm,
            entry_point: "main",
            compilation_options: Default::default(),
         });
      }


   }
}

fn load_shader(device: &Device, _map: String) -> std::thread::Result<ShaderModule> {
   // let source = include_str!("shaders/path_tracer.glsl").to_string();
   let source = std::fs::read_to_string("C:/Users/zacha/RustroverProjects/mgsdfe/rendering_stuff/src/shaders/tex_test.glsl").unwrap(); // for testing only

   let shader_mod = ShaderModuleDescriptor {
      label: None,
      source: ShaderSource::Glsl {
         shader: Cow::Owned(source),
         stage: ShaderStage::Compute,
         defines: FastHashMap::default(), // Adjust as needed for your shader
      },
   };

   std::panic::set_hook(Box::new(|panic_info| {
      if let Some(_) = panic_info.payload().downcast_ref::<&str>() {
         eprintln!("Panic occurred");
      } else {
         eprintln!("Panic occurred");
      }
   }));

   let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
      device.create_shader_module(shader_mod)
   }));

   if let Ok(_) = out {
      println!("Compiled")
   }

   out
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