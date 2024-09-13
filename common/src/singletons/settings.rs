use eframe::egui::Context;
use crate::init_none_static;

init_none_static!(SETTINGS: Settings);


pub struct Settings {}
impl Settings {
   pub fn new(_ctx: &Context) -> Self {
      Self {}
   }

   pub fn test(&self) {}
}

