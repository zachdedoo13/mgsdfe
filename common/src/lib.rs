pub mod macros;


/// shader graph data singleton
init_static!(SHADER_GRAPH_DATA: ShaderGraphData => { ShaderGraphData::default() });

pub struct ShaderGraphData {
   pub update_listener: UpdateListener,
   pub shader_code: ShaderCode,
}
impl Default for ShaderGraphData {
   fn default() -> Self {
      Self {
         update_listener: Default::default(),
         shader_code: Default::default(),
      }
   }
}


pub struct UpdateListener {
   pub queue_buffer: bool,
   pub queue_compile: bool,
}
impl Default for UpdateListener {
   fn default() -> Self {
      Self {
         queue_buffer: true,
         queue_compile: true
      }
   }
}


pub struct ShaderCode {
   pub code: String,
   pub data: Vec<f32>,
}
impl Default for ShaderCode {
   fn default() -> Self {
      Self {
         code: Default::default(),
         data: Default::default(),
      }
   }
}