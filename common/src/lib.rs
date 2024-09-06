use instant::Instant;

pub mod macros;


// shader graph data singleton
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
         queue_compile: true,
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



// time package
init_static!(TIME: TimePackage => { TimePackage::new() });

const PAST_FPS_LIMIT: usize = 100;
pub struct TimePackage {
   pub fps: f64,
   pub past_fps: Vec<[f64; 2]>, // for graphing
   pub delta_time: f64,
   frame_counter: u32,

   pub start_time: Instant,
   last_frame: Instant,
   last_data_dump: Instant,
   past_delta_times: Vec<f64>,

   pub fps_update_interval: f64,
   pub fps_amount: usize,
}
impl TimePackage {
   pub fn new() -> Self {
      Self {
         fps: 0.0,
         past_fps: vec![],
         delta_time: 0.0,
         frame_counter: 0,

         start_time: Instant::now(),
         last_frame: Instant::now(),
         last_data_dump: Instant::now(),
         past_delta_times: vec![],

         fps_update_interval: 0.25,
         fps_amount: 100,
      }
   }

   pub fn update(&mut self) {
      self.delta_time = self.last_frame.elapsed().as_secs_f64();

      if self.past_delta_times.len() < PAST_FPS_LIMIT {
         self.past_delta_times.push(self.delta_time);
      }

      if self.last_data_dump.elapsed().as_secs_f64() > 1.0 - self.fps_update_interval {
         self.calc_ave_fps();
         self.last_data_dump = Instant::now();
      }

      self.last_frame = Instant::now();
      self.frame_counter += 1;
   }

   fn calc_ave_fps(&mut self) {
      let mut total = 0.0;
      for num in &self.past_delta_times {
         total += num;
      }
      self.fps = 1.0 / (total / self.past_delta_times.len() as f64);
      self.past_delta_times.clear();
      self.past_fps.push([self.frame_counter as f64, self.fps]);

      let diff = self.past_fps.len() as i32 - self.fps_amount as i32;

      if diff > 0 {
         self.past_fps.drain(0..(diff as usize));
      }
   }
}