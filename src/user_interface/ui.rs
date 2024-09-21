use eframe::{CreationContext, Storage};
use egui::{CentralPanel, CollapsingHeader, ComboBox, DragValue, FontId, RichText, ScrollArea, SidePanel, Slider, TopBottomPanel, Ui, Vec2b};
use egui_plot::{Corner, Legend, Line, Plot};
use serde_json::{from_str, to_string};

use crate::{get, get_mut, get_mut_ref};
use crate::app::MgsApp;
use crate::singletons::settings::SETTINGS;
use crate::singletons::time_package::TIME;
use crate::user_interface::ui_modules::{enum_combination_box, ToggleSwitch};

#[derive(Copy, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
enum MainContentPage {
   NodeEditor,
   Stats,
   Settings,
}


/// user_interface state related data
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


/// main user_interface areas
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
               if ui.button("restart").clicked() { self.restart() };
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

// static mut TEST_THEME: catppuccin_egui::Theme = catppuccin_egui::MACCHIATO;

/// sub areas
impl MgsApp {
   fn stats(&mut self, ui: &mut Ui) {
      get_mut_ref!(SETTINGS, settings);
      let graph_set = &mut settings.graph_settings;

      let mw = per_width(ui, 0.25);
      ui.set_max_width(mw);


      const DEF_WIDTH: f32 = 600.0;
      const DEF_HEIGHT: f32 = 400.0;


      // fps graph
      ui.group(|ui| {
         let fps_graph_set = &mut graph_set.fps_graph_settings;


         ui.horizontal(|ui| {
            ui.heading("Application Fps");
            ui.label(format!("{}", get!(TIME).fps as i32));

            ui.menu_button("...", |ui| {
               if ui.button("Clear").clicked() { get_mut!(TIME).fps_graphing.clear() };

               if ui.add(Slider::new(&mut fps_graph_set.update_rate, 0.0..=0.99).text("Fps update rate")).changed() {
                  get_mut!(TIME).fps_update_interval = fps_graph_set.update_rate.sqrt();
               }

               if ui.add(Slider::new(&mut fps_graph_set.amount, 10..=500).text("Fps graph amount")).changed() {
                  get_mut!(TIME).fps_amount = fps_graph_set.amount;
               }
               ui.add(Slider::new(&mut fps_graph_set.include_upper, 0.0..=2500.0).text("Include at least")).changed();
            });
         });

         let data = get!(TIME).fps_graphing.clone();

         let line = Line::new(data).fill(0.0);
         Plot::new("my_plot")
             .view_aspect(2.0)
             .height(DEF_HEIGHT)
             .width(DEF_WIDTH)
             .allow_drag(false)
             .allow_scroll(false)
             .allow_zoom(false)
             .allow_boxed_zoom(false)
             .include_y(0.0)
             .include_y(fps_graph_set.include_upper)
             .show_axes(Vec2b::new(false, true))
             .show(ui, |plot_ui| plot_ui.line(line));
      });

      // gpu profile graph
      if cfg!(not(target_arch = "wasm32")) {

      }
      else {

      }

      ui.group(|ui| {
         if cfg!(not(target_arch = "wasm32")) {
            let gpu_set = &mut graph_set.gpu_profiler_graph_settings;
            let gpu_profiler = &mut self.path_tracer.gpu_profiler;

            gpu_profiler.amount = gpu_set.amount as u32;
            gpu_profiler.update_interval = 1.0 - gpu_set.update_rate;

            ui.horizontal(|ui| {
               ui.heading("Gpu performance profile");

               ui.menu_button("...", |ui| {
                  if ui.button("Clear").clicked() { gpu_profiler.timers.iter_mut().for_each(|e| e.1.time_graphing.clear()) };

                  ui.add(Slider::new(&mut gpu_set.update_rate, 0.01..=0.99).text("Update rate"));

                  ui.add(Slider::new(&mut gpu_set.amount, 10..=500).text("Graph amount"));

                  ui.add(Slider::new(&mut gpu_set.include_upper, 0.0..=5.0).text("Include at least"));

                  ui.add(Slider::new(&mut gpu_profiler.max_cash, 1..=50).text("Max cash, adjust for performance"));
               });

               ui.add_space(10.0);
               ui.add(ToggleSwitch::new(&mut self.path_tracer.do_gpu_profiling));
               ui.label("Active")
            });


            let mut data_entry = vec![];
            for e in gpu_profiler.timers.iter() {
               let data = &e.1.time_graphing;

               let first = data.last().unwrap_or(&[0.0, 0.0])[1];

               let label = format!("{:.2}ms => {}", first, e.0);

               data_entry.push(
                  Line::new(data.clone())
                      .name(label)
                      .fill(0.0)
               )
            }


            let plot = Plot::new("test_plot")
                .view_aspect(2.0)
                .height(DEF_HEIGHT)
                .width(DEF_WIDTH)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_boxed_zoom(false)
                .include_y(0.0)
                .include_y(gpu_set.include_upper)
                .y_axis_label("milliseconds")
                .show_axes(Vec2b::new(false, true))
                .legend(Legend::default().position(Corner::LeftBottom));

            plot.show(ui, |plot_ui| {
               for line in data_entry {
                  plot_ui.line(line);
               }
            });
         }
         else {
            ui.label("Gpu profiler not available on web builds");
         }
      });

   }

   fn settings_page(&mut self, ui: &mut Ui) {
      get_mut_ref!(SETTINGS, settings);

      // theme
      ui.group(|ui| {
         ui.label("Theme");
         if enum_combination_box(ui, &mut settings.theme, "Theme") { settings.theme.set_theme(ui.ctx()) };

         // unsafe {
         //    theme_color_picker(ui, &mut TEST_THEME);
         //    catppuccin_egui::set_theme(ui.ctx(), TEST_THEME);
         // }
      });
      ui.add_space(10.0);

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
            } else if z > upper {
               z = upper
            }

            ui.ctx().set_zoom_factor(z);
         });
      });
      ui.add_space(10.0);

      // dev settings
      ui.group(|ui| {
         CollapsingHeader::new("Developer Settings")
             .show(ui, |ui| {
                let ctx = ui.ctx().clone();
                ctx.settings_ui(ui);
             });
      });
      ui.add_space(10.0);
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

#[allow(dead_code)]
pub fn theme_color_picker(ui: &mut Ui, theme: &mut catppuccin_egui::Theme) {
   ui.color_edit_button_srgba(&mut theme.rosewater);
   ui.color_edit_button_srgba(&mut theme.flamingo);
   ui.color_edit_button_srgba(&mut theme.pink);
   ui.color_edit_button_srgba(&mut theme.mauve);
   ui.color_edit_button_srgba(&mut theme.red);
   ui.color_edit_button_srgba(&mut theme.maroon);
   ui.color_edit_button_srgba(&mut theme.peach);
   ui.color_edit_button_srgba(&mut theme.yellow);
   ui.color_edit_button_srgba(&mut theme.green);
   ui.color_edit_button_srgba(&mut theme.teal);
   ui.color_edit_button_srgba(&mut theme.sky);
   ui.color_edit_button_srgba(&mut theme.sapphire);
   ui.color_edit_button_srgba(&mut theme.blue);
   ui.color_edit_button_srgba(&mut theme.lavender);
   ui.color_edit_button_srgba(&mut theme.text);
   ui.color_edit_button_srgba(&mut theme.subtext1);
   ui.color_edit_button_srgba(&mut theme.subtext0);
   ui.color_edit_button_srgba(&mut theme.overlay2);
   ui.color_edit_button_srgba(&mut theme.overlay1);
   ui.color_edit_button_srgba(&mut theme.overlay0);
   ui.color_edit_button_srgba(&mut theme.surface2);
   ui.color_edit_button_srgba(&mut theme.surface1);
   ui.color_edit_button_srgba(&mut theme.surface0);
   ui.color_edit_button_srgba(&mut theme.base);
   ui.color_edit_button_srgba(&mut theme.mantle);
   ui.color_edit_button_srgba(&mut theme.crust);
}