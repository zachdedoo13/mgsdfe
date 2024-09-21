use std::borrow::Cow;
use std::panic::AssertUnwindSafe;
use std::thread;

use egui_wgpu::RenderState;
use log::error;
use wgpu::{CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource};
use wgpu::naga::{FastHashMap, ShaderStage};
use crate::gpu_profile_section;
use crate::path_tracer::render_utility::dual_storage_texture_package::DualStorageTexturePackage;
use crate::path_tracer::render_utility::gpu_profiler::GpuProfiler;
use crate::path_tracer::render_utility::helper_structs::UniformFactory;
use crate::singletons::scene::ParthtracerSettings;

pub struct PathTracerPackage {
   pub pipeline_layout: PipelineLayout,
   pub compute_pipeline: ComputePipeline,
   pub storage_textures: DualStorageTexturePackage,
   pub uniform: UniformFactory<ParthtracerSettings>,
}

impl PathTracerPackage {
   /// # Panics
   pub fn new(render_state: &RenderState, parthtracer_settings: &ParthtracerSettings) -> Self {
      let device = &render_state.device;

      let storage_textures = DualStorageTexturePackage::new(device);

      // todo placeholder
      let shader_module = load_shader(device, &String::new()).expect("Failed to load shader");

      let uniform = UniformFactory::new(device, parthtracer_settings);

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
         compilation_options: PipelineCompilationOptions::default(),
      });

      Self {
         pipeline_layout,
         compute_pipeline,
         storage_textures,
         uniform,
      }
   }

   pub fn update(&mut self, render_state: &RenderState, settings: ParthtracerSettings) {
      self.uniform.update_with_data(&render_state.queue, &settings);
   }

   pub fn render_pass(&mut self, encoder: &mut CommandEncoder, gpu_profiler: &mut GpuProfiler) {
      gpu_profile_section!(gpu_profiler, encoder, "SUB_PATHTRACE_PASS", {
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
      });

      // Perform the flip after the immutable borrows are done
      self.storage_textures.textures.flip();
   }

   pub fn remake_pipeline(&mut self, device: &Device) {
      let shader_module = load_shader(device, &String::new());

      if let Ok(sm) = shader_module {
         self.compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("path_tracer_pipeline"),
            layout: Some(&self.pipeline_layout),
            module: &sm,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
         });
      }
   }
}


fn load_shader(device: &Device, map: &String) -> thread::Result<ShaderModule> {
   load_shader_wgpu(device, map)
   // load_shader_with_naga(device)
}

fn load_shader_wgpu(device: &Device, _map: &String) -> thread::Result<ShaderModule> {
   // todo placeholder
   // let mapped_code = include_str!("shaders/testing.glsl").to_string();
   // let mapped_code = std::fs::read_to_string("path_tracer/src/shaders/testing.glsl").unwrap();
   let mapped_code = include_str!("shaders/testing.glsl").to_string();

   let source = mapped_code;

   let shader_mod = ShaderModuleDescriptor {
      label: None,
      source: ShaderSource::Glsl {
         shader: Cow::Owned(source),
         stage: ShaderStage::Compute,
         defines: FastHashMap::default(),
      },
   };

   let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
      let prev_hook = std::panic::take_hook();
      std::panic::set_hook(Box::new(|panic_info| {
         if let Some(e) = panic_info.payload().downcast_ref::<&str>() {
            error!("Panic occurred with e {e}");
         } else {
            error!("Panic occurred ");
         }
         error!("Occurred during the shader compilation");
      }));
      let result = device.create_shader_module(shader_mod);

      std::panic::set_hook(prev_hook);

      result
   }));

   if out.is_ok() {
      println!("Compiled");
   }

   out
}
