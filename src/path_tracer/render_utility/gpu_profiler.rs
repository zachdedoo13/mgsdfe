#![allow(unused_variables, dead_code)]

use egui::ahash::HashMap;
use instant::Instant;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, QuerySet, QuerySetDescriptor, QueryType, Queue};

pub struct GpuProfiler {
   pub query_count: u32,
   pub timers: HashMap<String, TimerEntry>,

   count: u32,
   buffer_size: u64,

   pub query_set: QuerySet,
   pub query_buffer: Buffer,
   pub cpu_buffer: Buffer,

   pub active: bool,

   pub amount: u32,
   pub update_interval: f64,
   pub max_cash: u32,
   last_data_dump: Instant,
}
impl GpuProfiler {
   pub fn new(device: &Device, query_count: u32) -> Self {
      let query_set = device.create_query_set(&QuerySetDescriptor {
         label: Some("Timestamp query"),
         ty: QueryType::Timestamp,
         count: query_count,
      });

      let (query_buffer, cpu_buffer, buffer_size) = Self::init_buffers(device, query_count);

      Self {
         query_count,
         timers: HashMap::default(),
         count: 0,
         active: true,
         buffer_size,
         query_set,
         query_buffer,
         cpu_buffer,

         amount: 50,
         update_interval: 0.25,
         max_cash: 25,
         last_data_dump: Instant::now(),
      }
   }

   fn init_buffers(device: &Device, count: u32) -> (Buffer, Buffer, u64) {
      let buffer_size = 8 * count as u64;

      let query_buffer = device.create_buffer(&BufferDescriptor {
         label: Some("query_buffer"),
         size: buffer_size,
         usage: BufferUsages::QUERY_RESOLVE |
             BufferUsages::STORAGE |
             BufferUsages::COPY_DST |
             BufferUsages::COPY_SRC,
         mapped_at_creation: false,
      });

      let cpu_buffer = device.create_buffer(&BufferDescriptor {
         label: Some("query_buffer"),
         size: buffer_size,
         usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
         mapped_at_creation: false,
      });

      (query_buffer, cpu_buffer, buffer_size)
   }

   pub fn start_timer(&mut self, encoder: &mut CommandEncoder, key: &str) {
      #[cfg(not(target_arch = "wasm32"))]
      if self.active {
         if self.count < self.query_count {
            encoder.write_timestamp(&self.query_set, self.count);

            self.check_add_timer_entry(key);
            self.timers.get_mut(key).unwrap().start_index = self.count;

            self.count += 1;
         } else {
            panic!("Warning: Query count exceeded the available queries.");
         }
      }
   }

   pub fn end_timer(&mut self, encoder: &mut CommandEncoder, key: &str) {
      #[cfg(not(target_arch = "wasm32"))]
      if self.active {
         if self.count < self.query_count {
            encoder.write_timestamp(&self.query_set, self.count);

            self.check_add_timer_entry(key);
            self.timers.get_mut(key).unwrap().end_index = self.count;

            self.count += 1;
         } else {
            panic!("Warning: Query count exceeded the available queries.");
         }
      }
   }

   fn check_add_timer_entry(&mut self, key: &str) {
      if !self.timers.contains_key(key) { self.timers.insert(key.to_string(), TimerEntry::default()); };
   }

   pub fn resolve(&mut self, encoder: &mut CommandEncoder, device: &Device) {
      #[cfg(not(target_arch = "wasm32"))]
      if self.active {
         encoder.resolve_query_set(&self.query_set, 0..self.query_count, &self.query_buffer, 0);

         encoder.copy_buffer_to_buffer(&self.query_buffer, 0, &self.cpu_buffer, 0, self.buffer_size);

         if self.count < self.query_count {
            println!("Change query count on gpu profiler to {}", self.count);
            *self = Self::new(device, self.count);
         }

         self.count = 0;
      }
   }

   fn read_data(&mut self, queue: &Queue, device: &Device) -> Vec<(String, f64)> {
      let timeings: Vec<u64> = read_buffer_to_vec(&device, &self.cpu_buffer).expect("failed to read data");
      let period = queue.get_timestamp_period() as f64;

      let mut out = vec![];
      for e in self.timers.iter() {
         let te = e.1;

         let st = timeings[te.start_index as usize];
         let et = timeings[te.end_index as usize];

         let time = (et - st) as f64 * period;

         let time_ms = nanoseconds_to_milliseconds(time);

         out.push((e.0.clone(), time_ms));
      }

      out
   }

   pub fn update(&mut self, queue: &Queue, device: &Device) {
      if self.active {
         let timeings = self.read_data(queue, device);

         timeings.iter().for_each(|e| {
            self.timers.get_mut(e.0.as_str()).unwrap().add_cash(e.1, self.max_cash);
         });

         if self.last_data_dump.elapsed().as_secs_f64() > self.update_interval {
            self.last_data_dump = Instant::now();

            for timer in self.timers.iter_mut() {
               timer.1.calc_ave(self.amount);
            }
         }
      }
   }
}


/// edit for performance, smooths out average

pub struct TimerEntry {
   start_index: u32,
   end_index: u32,
   elapsed_cash: Vec<f64>,
   index_counter: u32,

   pub time_graphing: Vec<[f64; 2]>,
}
impl TimerEntry {
   pub fn add_cash(&mut self, data: f64, max_cash: u32) {
      if self.elapsed_cash.len() < max_cash as usize {
         self.elapsed_cash.push(data);
      }
   }

   pub fn calc_ave(&mut self, max_cash: u32) {
      let mut total = 0.0;
      for num in &self.elapsed_cash {
         total += num;
      }

      let ave = total / self.elapsed_cash.len() as f64;

      self.elapsed_cash.clear();

      self.time_graphing.push([self.index_counter as f64, ave]);
      self.index_counter += 1;

      let diff = self.time_graphing.len() as i32 - max_cash as i32;

      if diff > 0 {
         self.time_graphing.drain(0..(diff as usize));
      }
   }
}

impl Default for TimerEntry {
   fn default() -> Self {
      Self {
         start_index: 0,
         end_index: 0,
         elapsed_cash: vec![],
         index_counter: 0,
         time_graphing: vec![],
      }
   }
}

pub fn read_buffer_to_vec<T: bytemuck::Pod>(device: &Device, buffer: &Buffer) -> Option<Vec<T>> {
   let buffer_slice = buffer.slice(..);
   let (sender, receiver) = flume::bounded(1);
   buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

   device.poll(wgpu::Maintain::wait()).panic_on_timeout();

   let future = receiver.recv_async();

   if let Ok(Ok(())) = pollster::block_on(future) {
      let data = buffer_slice.get_mapped_range();

      let out_data = bytemuck::cast_slice(&data).to_vec();


      drop(data);
      buffer.unmap(); // check if this dose anything

      return Some(out_data);
   };

   None
}

pub fn nanoseconds_to_milliseconds(nanoseconds: f64) -> f64 {
   nanoseconds / 1_000_000.0
}