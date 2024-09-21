use egui::ahash::HashMap;
use egui_wgpu::RenderState;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, QuerySet, QuerySetDescriptor, QueryType};

pub struct GpuProfiler {
   pub query_count: u32,
   pub timers: HashMap<String, (u32, u32)>,

   count: u32,
   buffer_size: u64,

   pub query_set: QuerySet,
   pub query_buffer: Buffer,
   pub cpu_buffer: Buffer,
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
         buffer_size,
         query_set,
         query_buffer,
         cpu_buffer,
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

   pub fn init_timer(&mut self, encoder: &mut CommandEncoder, name: &str) {
      {
         self.timers.insert(name.to_string(), (self.count, u32::MAX));

         encoder.write_timestamp(&self.query_set, self.count);

         self.count += 1;
      }
   }

   pub fn end_timer(&mut self, encoder: &mut CommandEncoder, name: &str) {
      {
         let current = self.timers.get(name).expect("No corresponding start timer");
         self.timers.insert(name.to_string(), (current.0, self.count));

         encoder.write_timestamp(&self.query_set, self.count);

         self.count += 1;
      }
   }

   pub fn resolve(&mut self, encoder: &mut CommandEncoder) {
      encoder.resolve_query_set(&self.query_set, 0..self.query_count, &self.query_buffer, 0);

      encoder.copy_buffer_to_buffer(&self.query_buffer, 0, &self.cpu_buffer, 0, self.buffer_size);
   }

   pub fn read_data(&mut self, render_state: &RenderState) {
      let timeings: Vec<u64> = read_buffer_to_vec(&render_state.device, &self.cpu_buffer).expect("failed to read data");
      let period = render_state.queue.get_timestamp_period() as f64;

      for e in self.timers.iter() {
         let (si, ei) = e.1;

         let st = timeings[*si as usize];
         let et = timeings[*ei as usize];

         let time = (et - st) as f64 * period;
         println!("{} => {}ns", e.0, time);
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

      buffer.unmap(); // check if this dose anything

      return Some(out_data);
   };

   None
}