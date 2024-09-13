use eframe::egui::Context;
use crate::init_none_static;

init_none_static!(SETTINGS: Settings);

/// global settings for the app
pub struct Settings {}
impl Settings {
   pub fn new(_ctx: &Context) -> Self {
      // load self from persistent storage
      Self {}
   }

   pub fn test(&self) {}
}

impl Default for Settings {
   fn default() -> Self {
      Self {}
   }
}

