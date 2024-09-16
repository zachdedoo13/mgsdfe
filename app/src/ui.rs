use std::fmt::Debug;

use eframe::{CreationContext, Storage};
use egui::{CentralPanel, ComboBox, FontId, RichText, ScrollArea, SidePanel, TopBottomPanel, Ui, Vec2b};
use egui_plot::{Line, Plot};
use serde_json::{from_str, to_string};
use strum::IntoEnumIterator;

use common::{get, get_mut_ref};
use common::singletons::settings::SETTINGS;
use common::singletons::time_package::TIME;

use crate::MgsApp;

#[derive(Copy, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
enum MainContentPage {
   NodeEditor,
   Stats,
   Settings,
}


/// ui state related data
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct UiState {
   main_content_page: MainContentPage,
}
impl UiState {
   pub fn new(cc: &CreationContext) -> Self {
      let str = cc.storage.unwrap().get_string("ui_state");
      match str {
         None => UiState::default(),
         Some(str) => from_str::<UiState>(str.as_str()).unwrap(),
      }
   }

   pub fn save(&self, storage: &mut dyn Storage) {
      storage.set_string("ui_state", to_string(self).unwrap());
   }
}
impl Default for UiState {
   fn default() -> Self {
      Self {
         main_content_page: MainContentPage::NodeEditor,
      }
   }
}


/// main ui areas
impl MgsApp {
   /// handles sectioning
   pub fn ui(&mut self, ui: &mut Ui) {
      self.top_menubar(ui);

      SidePanel::left("Left menubar")
          .default_width(0.0)
          .show_inside(ui, |ui| {
             ui.horizontal(|ui| {
                ui.vertical(|ui| {
                   ui.add_space(25.0);
                   self.left_navigation(ui);
                });
                ui.add_space(2.0);
             });
          });

      SidePanel::right("Viewport and tracer settings")
          .resizable(true)
          .min_width(per_width(ui, 0.2))
          .max_width(per_width(ui, 0.70))
          .show_inside(ui, |ui| {
             TopBottomPanel::top("tracer settings")
                 .show_inside(ui, |ui| {
                    self.path_tracer(ui);
                 });

             CentralPanel::default()
                 .show_inside(ui, |ui| {
                    self.tracer_settings(ui);
                 });
          });

      CentralPanel::default()
          .show_inside(ui, |ui| {
             self.main_content(ui);
          });
   }

   fn top_menubar(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
               if ui.button("test").clicked() {};
               if ui.button("test2").clicked() {};
            });

            ui.menu_button("Edit", |ui| {
               if ui.button("test").clicked() {};
               if ui.button("test2").clicked() {};
            });
         });
      });
   }

   fn left_navigation(&mut self, ui: &mut Ui) {
      const SPACE: f32 = 15.0;
      let large_emoji = |e: &str| -> RichText {
         RichText::new(e).font(FontId::proportional(30.0))
      };

      if ui.button(large_emoji("📝")).clicked() {
         self.ui_state.main_content_page = MainContentPage::NodeEditor;
      }
      ui.add_space(SPACE);

      if ui.button(large_emoji("📊")).clicked() {
         self.ui_state.main_content_page = MainContentPage::Stats;
      }
      ui.add_space(SPACE);

      if ui.button(large_emoji("🔧")).clicked() {
         self.ui_state.main_content_page = MainContentPage::Settings;
      }
      ui.add_space(SPACE);
   }

   fn path_tracer(&mut self, ui: &mut Ui) {
      self.path_tracer.display(ui);
   }

   fn tracer_settings(&mut self, _ui: &mut Ui) {}

   fn main_content(&mut self, ui: &mut Ui) {
      match self.ui_state.main_content_page {
         MainContentPage::NodeEditor => {
            ui.label("node editor");
         }

         MainContentPage::Stats => {
            ScrollArea::vertical()
                .show(ui, |ui| {
                   self.stats(ui);

                   // moves scroll bar to the right
                   ui.set_min_width(ui.available_size().x)
                });
         }

         MainContentPage::Settings => {
            self.settings_page(ui);
         }
      }
   }
}

/// sub areas
impl MgsApp {
   fn stats(&mut self, ui: &mut Ui) {
      // fps graph
      ui.group(|ui| {
         let mw = per_width(ui, 0.25);
         ui.set_max_width(mw);

         ui.horizontal(|ui| {
            ui.heading("Overall fps");
            ui.label(format!("{}", get!(TIME).fps as i32));
         });

         let mut data = get!(TIME).past_fps.clone();
         if data.len() > 0 { data.insert(0, [data[0][0] - 1.0, 0.0]); } // makes the zoom include {y: 0}

         let line = Line::new(data);
         Plot::new("my_plot")
             .view_aspect(2.0)
             .allow_drag(false)
             .allow_scroll(false)
             .allow_zoom(false)
             .allow_boxed_zoom(false)
             .show_axes(Vec2b::new(false, true))
             .show(ui, |plot_ui| plot_ui.line(line));
      });
   }

   fn settings_page(&mut self, ui: &mut Ui) {
      get_mut_ref!(SETTINGS, settings);

      // theme
      ui.group(|ui| {
         ui.label("Theme");
         if enum_combination_box(ui, &mut settings.theme, "") { settings.theme.set_theme(ui.ctx()) };
      });
   }
}

/////////////////////////////
// miscellaneous functions //
/////////////////////////////
fn per_width(ui: &mut Ui, per: f32) -> f32 {
   ui.ctx().screen_rect().width() * per
}

/// returns a true if changed
pub fn enum_combination_box<T, I>(ui: &mut Ui, combination_type: &mut T, label: I) -> bool
where
    T: IntoEnumIterator + Debug + PartialEq + Copy,
    I: Into<String>,
{
   let mut changed = false;
   ComboBox::from_label(label.into().as_str())
       .selected_text(format!("{combination_type:?}"))
       .show_ui(ui, |ui| {
          for variant in T::iter() {
             if ui.selectable_value(combination_type, variant, format!("{variant:?}")).changed() { changed = true; }
          }
       });

   changed
}