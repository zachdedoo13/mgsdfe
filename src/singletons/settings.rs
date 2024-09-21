use catppuccin_egui::{FRAPPE, LATTE, MACCHIATO, MOCHA};
use eframe::{CreationContext, Storage};
use eframe::egui::{Context, Visuals};
use serde_json::{from_str, to_string};
use strum::{Display, EnumIter};

use crate::init_none_static;
use crate::singletons::scene::Scene;

init_none_static!(SETTINGS: Settings);

/// global settings for the app, init in ``App::new()`` and saved in ``App::save()``
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Settings {
   pub theme: Theme,

   pub saved_scenes: Vec<Scene>,
   pub current_scene: Scene,

   pub image_size_settings: ImageSizeSettings,

   pub graph_settings: GraphSettings,
}

impl Settings {
   /// init loading from context
   /// # Panics
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

   /// # Panics
   pub fn save(&self, storage: &mut dyn Storage) {
      storage.set_string("settings", to_string(self).unwrap());
   }
}

impl Default for Settings {
   fn default() -> Self {
      Self {
         theme: Theme::Dark,
         saved_scenes: vec![],
         current_scene: Scene::default(),
         image_size_settings: ImageSizeSettings::default(),
         graph_settings: GraphSettings::default(),
      }
   }
}


////////////////////
// Theme settings //
////////////////////
#[derive(PartialEq, Clone, Debug, Copy, EnumIter, Display)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Theme {
   Dark,
   Light,
   Latte,
   Frappe,
   Macchiato,
   Mocha,
   Oled,
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
         Theme::Oled => { todo!() },
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

   pub sampling_type: SamplingType,
}

impl Default for ImageSizeSettings {
   fn default() -> Self {
      Self {
         maintain_aspect_ratio: true,
         selected_aspect: (16, 9),
         aspect_scale: 1920,

         width: 1920,
         height: 1080,

         sampling_type: SamplingType::Biliniur,
      }
   }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, EnumIter, Debug, PartialEq)]
pub enum SamplingType {
   Biliniur,
   Linear,
}


////////////////////
// Graph settings //
////////////////////
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct GraphSettings {
   pub fps_graph_settings: FpsGraphSettings,
}

impl Default for GraphSettings {
   fn default() -> Self {
      Self {
         fps_graph_settings: FpsGraphSettings {
            include_upper: 200.0,
            update_rate: 0.25,
            amount: 100,
         }
      }
   }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct FpsGraphSettings {
   pub include_upper: f32,
   pub update_rate: f64,
   pub amount: usize,
}