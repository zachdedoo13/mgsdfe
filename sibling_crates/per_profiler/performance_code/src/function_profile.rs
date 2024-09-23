use instant::Instant;

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