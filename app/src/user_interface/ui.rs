use std::fmt::Debug;

use eframe::{CreationContext, Storage};
use egui::{CentralPanel, CollapsingHeader, ComboBox, DragValue, FontId, Response, RichText, ScrollArea, SidePanel, Slider, TopBottomPanel, Ui, Vec2b};
use egui_plot::{Line, Plot};
use serde_json::{from_str, to_string};
use strum::IntoEnumIterator;

use common::{get, get_mut, get_mut_ref};
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
   /// # Panics
   pub fn new(cc: &CreationContext) -> Self {
      let str = cc.storage.unwrap().get_string("ui_state");
      match str {
         None => UiState::default(),
         Some(str) => from_str::<UiState>(str.as_str()).unwrap(),
      }
   }

   /// # Panics
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
             TopBottomPanel::bottom("tracer settings")
                 .resizable(true)
                 .show_inside(ui, |ui| {
                    self.tracer_settings(ui);
                    ui.set_min_height(ui.available_size().y);
                 });

             CentralPanel::default()
                 .show_inside(ui, |ui| {
                    self.path_tracer(ui);
                    ui.add_space(50.0);
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

   fn tracer_settings(&mut self, ui: &mut Ui) {
      ScrollArea::vertical().show(ui, |ui| {
         ui.add_space(10.0);

         ui.horizontal(|ui| {
            ui.vertical(|ui| {
               get_mut!(SETTINGS).current_scene.parthtrace_settings.ui(ui);
            });

            ui.vertical(|ui| {
               self.image_render_settings(ui);
            });
         });

         ui.set_min_width(ui.available_width());
      });
   }

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
            ScrollArea::vertical()
                .show(ui, |ui| {
                   self.settings_page(ui);

                   // moves scroll bar to the right
                   ui.set_min_width(ui.available_size().x)
                });
         }
      }
   }
}

/// sub areas
impl MgsApp {
   fn stats(&mut self, ui: &mut Ui) {
      get_mut_ref!(SETTINGS, settings);
      let graph_set = &mut settings.graph_settings;
      let fps_graph_set = &mut graph_set.fps_graph_settings;

      // fps graph
      ui.group(|ui| {
         let mw = per_width(ui, 0.25);
         ui.set_max_width(mw);

         ui.horizontal(|ui| {
            ui.heading("Application Fps");
            ui.label(format!("{}", get!(TIME).fps as i32));

            ui.menu_button("...", |ui| {
               if ui.button("Clear").clicked() { get_mut!(TIME).past_fps.clear() };

               if ui.add(Slider::new(&mut fps_graph_set.update_rate, 0.0..=0.99).text("Fps update rate")).changed() {
                  get_mut!(TIME).fps_update_interval = fps_graph_set.update_rate.sqrt();
               }

               if ui.add(Slider::new(&mut fps_graph_set.amount, 10..=250).text("Fps graph amount")).changed() {
                  get_mut!(TIME).fps_amount = fps_graph_set.amount;
               }
               ui.add(Slider::new(&mut fps_graph_set.include_upper, 0.0..=1000.0).text("Include at least")).changed();
            });
         });

         let data = get!(TIME).past_fps.clone();

         let line = Line::new(data);
         Plot::new("my_plot")
             .view_aspect(2.0)
             .allow_drag(false)
             .allow_scroll(false)
             .allow_zoom(false)
             .allow_boxed_zoom(false)
             .include_y(0.0)
             .include_y(fps_graph_set.include_upper)
             .show_axes(Vec2b::new(false, true))
             .show(ui, |plot_ui| plot_ui.line(line));
      });
   }

   fn settings_page(&mut self, ui: &mut Ui) {
      get_mut_ref!(SETTINGS, settings);

      // theme
      ui.group(|ui| {
         ui.label("Theme");
         if enum_combination_box(ui, &mut settings.theme, "Theme") { settings.theme.set_theme(ui.ctx()) };
      });

      // Zoom
      ui.group(|ui| {
         let step = 0.05;
         let lower = 0.5;
         let upper = 2.0;

         ui.label("Zoom");

         ui.horizontal(|ui| {
            let un_rounded = ui.ctx().zoom_factor();
            let mut z = (un_rounded * 100.0).round() / 100.0;

            if ui.button("-").clicked() {
               z -= step;
            }

            ui.add(DragValue::new(&mut z).speed(0.001).range(lower..=upper).suffix("%"));

            if ui.button("+").clicked() {
               z += step;
            }

            ui.add_space(5.0);
            if ui.button("🔄").clicked() {
               z = 1.0;
            }


            if z < lower {
               z = lower
            }
            else if z > upper {
               z = upper
            }

            ui.ctx().set_zoom_factor(z);
         });
      });

      // dev settings
      ui.group(|ui| {
         CollapsingHeader::new("Developer Settings")
             .show(ui, |ui| {
                let ctx = ui.ctx().clone();
                ctx.settings_ui(ui);
             });
      });
   }

   fn image_render_settings(&mut self, ui: &mut Ui) {
      get_mut_ref!(SETTINGS, settings);
      let iss = &mut settings.image_size_settings;

      ui.group(|ui| {
         ui.horizontal(|ui| {
            ui.label("Resolution settings");
         });

         ui.group(|ui| {
            ui.horizontal(|ui| {
               ui.add(ToggleSwitch::new(&mut iss.maintain_aspect_ratio));

               ui.label("Maintain aspect");
            });
         }).response.on_hover_text("Whether the image expands to fit available space regardless of aspect or forces correct aspect");

         enum_combination_box(ui, &mut iss.sampling_type, "Sampling type");

         {
            let aspect_ratios = vec![
               (16, 9),
               (4, 3),
               (21, 9),
               (1, 1),
            ];

            ComboBox::from_label("Aspect Ratio")
                .selected_text(format!("{}:{}", iss.selected_aspect.0, iss.selected_aspect.1))
                .show_ui(ui, |ui| {
                   for aspect in &aspect_ratios {
                      ui.selectable_value(&mut iss.selected_aspect, *aspect, format!("{}:{}", aspect.0, aspect.1));
                   }
                });

            let aspect = iss.selected_aspect.1 as f32 / iss.selected_aspect.0 as f32;
            ui.add(Slider::new(&mut iss.aspect_scale, 8..=3840).text("Scale"));

            let h = (iss.aspect_scale as f32 * aspect) as u32;
            let w = (iss.aspect_scale as f32) as u32;

            ui.group(|ui| {
               ui.label(format!("Dimensions => {w}x{h}"));
               ui.label(format!("Total pixels => {:.2} Million", (w * h) as f32 / 1_000_000.0));
            });

            iss.width = w;
            iss.height = h;
         }
      });
   }
}

/////////////////////////////
// miscellaneous functions //
/////////////////////////////
fn per_width(ui: &mut Ui, per: f32) -> f32 {
   ui.ctx().screen_rect().width() * per
}

//////////////////////////
// Ui related functions //
//////////////////////////

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

pub struct ToggleSwitch<'a> {
   on_off: &'a mut bool,
}

impl<'a> ToggleSwitch<'a> {
   pub fn new(val: &'a mut bool) -> Self {
      Self {
         on_off: val,
      }
   }
}

impl<'a> egui::Widget for ToggleSwitch<'a> {
   fn ui(self, ui: &mut Ui) -> Response {
      let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
      let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
      if response.clicked() {
         *self.on_off = !*self.on_off;
         response.mark_changed();
      }

      response.widget_info(|| {
         egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *self.on_off, "")
      });

      if ui.is_rect_visible(rect) {
         let how_on = ui.ctx().animate_bool_responsive(response.id, *self.on_off);
         let visuals = ui.style().interact_selectable(&response, *self.on_off);
         let rect = rect.expand(visuals.expansion);
         let radius = 0.5 * rect.height();
         ui.painter()
             .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
         let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
         let center = egui::pos2(circle_x, rect.center().y);
         ui.painter()
             .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
      }

      response
   }
}