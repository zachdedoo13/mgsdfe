use instant::Instant;



const UPDATE_INTERVAL: f64 = 0.25;
const PAST_FPS_LIMIT: usize = 1000;


pub struct TimePackage {
   pub fps: f64,
   pub past_fps: Vec<f64>,
   pub delta_time: f64,

   pub start_time: Instant,
   last_frame: Instant,
   last_data_dump: Instant,
   past_delta_times: Vec<f64>,
}
impl TimePackage {
   pub fn new() -> Self {
      Self {
         fps: 0.0,
         past_fps: vec![],
         delta_time: 0.0,

         start_time: Instant::now(),
         last_frame: Instant::now(),
         last_data_dump: Instant::now(),
         past_delta_times: vec![],
      }
   }

   pub fn update(&mut self) {
      self.delta_time = self.last_frame.elapsed().as_secs_f64();

      if self.past_delta_times.len() < PAST_FPS_LIMIT {
         self.past_delta_times.push(self.delta_time);
      }

      if self.last_data_dump.elapsed().as_secs_f64() > UPDATE_INTERVAL {
         self.calc_ave_fps();
         self.last_data_dump = Instant::now();
      }

      self.last_frame = Instant::now();
   }

   fn calc_ave_fps(&mut self) {
      let mut total = 0.0;
      for num in &self.past_delta_times {
         total += num;
      }
      self.fps = 1.0 / (total / self.past_delta_times.len() as f64);
      self.past_delta_times.clear();
      self.past_fps.push(self.fps);
      if self.past_fps.len() > 100 {
         self.past_fps.remove(0);
      }
   }
}
