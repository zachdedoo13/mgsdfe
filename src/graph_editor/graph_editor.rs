use eframe::Storage;

pub struct GraphEditor {}
impl GraphEditor {

   pub fn new() -> Self {
      Self {}
   }

   pub fn update(&mut self) {}

   pub fn ui(&mut self) {}

   pub fn save(&mut self, _storage: &mut dyn Storage) {}

}