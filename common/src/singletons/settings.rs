use catppuccin_egui::{FRAPPE, LATTE, MACCHIATO, MOCHA};
use eframe::egui::{Context, Visuals};
use eframe::{CreationContext, Storage};
use serde_json::{from_str, to_string};
use crate::init_none_static;
use crate::singletons::scene::Scene;

init_none_static!(SETTINGS: Settings);

/// global settings for the app, init in app::new and saved in app::save
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Settings {
   pub theme: Theme,

   pub saved_scenes: Vec<Scene>,
   pub current_scene: Scene,

   pub image_size_settings: ImageSizeSettings,
}
impl Settings {
   /// init loading from context
   pub fn new(cc: &CreationContext) -> Self {
      // load self from persistent storage
      let per = cc.storage.unwrap().get_string("settings");
      match per {
         None => Settings::default(),
         Some(str) => {
            let set: Settings = from_str(str.as_str()).unwrap_or_default();
            set.theme.set_theme(&cc.egui_ctx);
            set
         }
      }
   }

   pub fn save(&self, storage: &mut dyn Storage) {
      storage.set_string("settings", to_string(self).unwrap())
   }
}

impl Default for Settings {
   fn default() -> Self {
      Self {
         theme: Theme::Dark,
         saved_scenes: vec![],
         current_scene: Scene::default(),
         image_size_settings: ImageSizeSettings::default(),
      }
   }
}


////////////////////
// Theme settings //
////////////////////
#[derive(PartialEq, Clone, Debug, Copy, strum::EnumIter, strum::Display)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Theme {
   Dark,
   Light,
   Latte,
   Frappe,
   Macchiato,
   Mocha,
}
impl Theme {
   pub fn set_theme(&self, ctx: &Context) {
      match self {
         Theme::Dark => ctx.set_visuals(Visuals::dark()),
         Theme::Light => ctx.set_visuals(Visuals::light()),
         Theme::Latte => catppuccin_egui::set_theme(ctx, LATTE),
         Theme::Frappe => catppuccin_egui::set_theme(ctx, FRAPPE),
         Theme::Macchiato => catppuccin_egui::set_theme(ctx, MACCHIATO),
         Theme::Mocha => catppuccin_egui::set_theme(ctx, MOCHA),
      };
   }
}


/////////////////////////
// Image size settings //
/////////////////////////
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct ImageSizeSettings {
   pub maintain_aspect_ratio: bool,
   pub selected_aspect: (i32, i32),
   pub aspect_scale: i32,

   pub width: u32,
   pub height: u32,
}
impl Default for ImageSizeSettings {
   fn default() -> Self {
      Self {
         maintain_aspect_ratio: true,
         selected_aspect: (16, 9),
         aspect_scale: 1920,

         width: 1920,
         height: 1080,
      }
   }
}