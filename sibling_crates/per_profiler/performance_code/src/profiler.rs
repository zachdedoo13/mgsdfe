use std::collections::HashMap;
use std::time::Duration;
use instant::Instant;

/// basic first attempt
#[derive(Debug)]
pub struct PerformanceProfiler {
   pub active: bool,
   pub profiles: HashMap<String, FunctionProfile>,

   pub stored_data_amount: u32,
   pub stored_cash_amount: u32,
   pub update_interval: Duration,

   last_dump: Instant
}
impl Default for PerformanceProfiler {
   fn default() -> Self {
      Self {
         active: true,
         profiles: Default::default(),

         stored_data_amount: 100,
         stored_cash_amount: 10,
         update_interval: Duration::from_millis(250),
         last_dump: Instant::now(),
      }
   }
}

impl PerformanceProfiler {
   pub fn start_time_function<T: Into<String>>(&mut self, name: T) {
      if !self.active { return; }
      let name = name.into();

      match self.profiles.get_mut(name.as_str()) {
         None => {
            self.profiles.insert(name.clone(), FunctionProfile::default());
            self.start_time_function(name);
         },
         Some(profile) => {
            profile.start();
         }
      }
   }

   pub fn end_time_function<T: Into<String>>(&mut self, name: T) {
      if !self.active { return; }
      let name = name.into();

      match self.profiles.get_mut(name.as_str()) {
         None => {
            panic!("Timer ended without a start");
         },
         Some(profile) => {
            profile.end();
         }
      }
   }
}

impl PerformanceProfiler {
   pub fn resolve_profiler(&mut self) {
      if !self.active { return; }

      if self.last_dump.elapsed() > self.update_interval {

         self.last_dump = Instant::now();

         for (_, profile) in self.profiles.iter_mut() {
            profile.resolve(self.stored_cash_amount, self.stored_data_amount);
         }
      }
   }
}


#[derive(Debug)]
pub struct FunctionProfile {
   st: Instant,
   counter: u32,
   max_stored_cash_amount: u32,

   pub average_cash: Vec<f64>,

   /// [0] is an index, used for graphing with ``egui_graph``
   /// [1] is the actual time elapsed in ms
   pub timeings: Vec<[f64; 2]>,
}
impl Default for FunctionProfile {
   fn default() -> Self {
      Self {
         st: Instant::now(),
         counter: 0,
         max_stored_cash_amount: 10,
         average_cash: vec![],
         timeings: vec![],
      }
   }
}
impl FunctionProfile {
   pub fn start(&mut self) {
      if (self.average_cash.len() as u32) < self.max_stored_cash_amount {
         self.st = Instant::now();
      }

   }
   pub fn end(&mut self) {
      if (self.average_cash.len() as u32) < self.max_stored_cash_amount {
         self.average_cash.push(self.st.elapsed().as_secs_f64() * 1000.0);
      }
   }
   pub fn resolve(&mut self, stored_cash_amount: u32, stored_data_amount: u32) {
      self.max_stored_cash_amount = stored_cash_amount;

      let ave: f64 = self.average_cash.iter().sum::<f64>() / self.average_cash.len() as f64;

      self.timeings.push(
         [self.counter as f64, ave]
      );

      let diff = self.timeings.len() as i32 - stored_data_amount as i32;
      if diff > 0 { self.timeings.drain(0..(diff as usize)); }

      self.average_cash.clear();
      self.counter += 1;
   }
}