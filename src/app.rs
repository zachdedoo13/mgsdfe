use std::sync::{Mutex};
use std::time::Duration;
use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, ComboBox, Context, DragValue, ScrollArea, SidePanel, Slider, TopBottomPanel, Ui, Vec2b, Visuals};
use egui::panel::{Side, TopBottomSide};
use egui_plot::{Line, Plot};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use crate::{get, init_static, render_pack_from_frame};
use crate::packages::time_package::TimePackage;
use crate::render_state::meh_renderer::MehRenderer;
use crate::render_state::structs::RenderPack;


// Globals
init_static!(TIME: TimePackage => {TimePackage::new()});


// #[cfg(not(target_arch = "wasm32"))]
// init_static!(SYS: SysUsage => { SysUsage::new() }); // todo => current fucked
//
// #[cfg(not(target_arch = "wasm32"))]
// pub struct SysUsage {
//    machine: machine_info::Machine,
// }
// #[cfg(not(target_arch = "wasm32"))]
// impl SysUsage {
//    pub fn new() -> Self {
//       let mut machine = machine_info::Machine::new();
//       // let pid = std::process::id() as i32;
//
//       // machine.track_process(pid).unwrap();
//
//       Self {
//          machine,
//       }
//    }
// }



pub struct MehApp {
    meh_renderer: MehRenderer,
}
impl App for MehApp {
   fn update(&mut self, ctx: &Context, frame: &mut Frame) {
      render_pack_from_frame!(render_pack, frame);

      self.update(&mut render_pack);
      self.render(&render_pack);
      self.ui(ctx);

      ctx.request_repaint();
   }

   fn auto_save_interval(&self) -> Duration {
      Duration::from_secs_f32(1.0)
   }
}
impl MehApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let renderer = &mut render_state.renderer.write();
        let device = &render_state.device;


        let meh_renderer = MehRenderer::new(device, renderer);


        Self {
            meh_renderer,
        }
    }

    fn update(&mut self, render_pack: &mut RenderPack<'_>) {
        get!(TIME).update();

        self.meh_renderer.update(render_pack)
    }

    fn render(&mut self, render_pack: &RenderPack<'_>) {
        self.meh_renderer.render_pass(render_pack)

    }

    fn ui(&mut self, ctx: &Context) {
        CentralPanel::default()
            .show(ctx, |ui| {
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

                SidePanel::new(Side::Left, "Left main")
                    .resizable(true)
                    .default_width(500.0)
                    .show_inside(ui, |ui| {
                        // sdf editor

                       ui.group(|ui| { ui.label("big\nbad\nsdf\neditor"); });


                       ui.allocate_space(ui.available_size());
                    });


                CentralPanel::default()
                    .show_inside(ui, |ui| {
                        TopBottomPanel::new(TopBottomSide::Bottom, "bottom right")
                            .resizable(true)
                            .default_height(500.0)
                            .show_inside(ui, |ui| {

                                // settings
                               SidePanel::new(Side::Left, "left settings")
                                   .show_inside(ui, |ui| {
                                      ui.add_space(6.0);
                                      let mut temp = ctx.memory_mut(|mem| { mem.data.get_persisted("VarPage1".into()).unwrap_or_else(|| VariablePage::Shader) });
                                      variable_areas(ctx, ui, &mut temp);
                                      ctx.memory_mut( |mem| mem.data.insert_persisted("VarPage1".into(), temp.clone()) );

                                      ui.set_min_width(ui.available_size().x);
                                   });

                               // stats
                               CentralPanel::default()
                                   .show_inside(ui, |ui| {

                                      let mut temp = ctx.memory_mut(|mem| { mem.data.get_persisted("VarPage2".into()).unwrap_or_else(|| VariablePage::Stats) });
                                      variable_areas(ctx, ui, &mut temp);
                                      ctx.memory_mut( |mem| mem.data.insert_persisted("VarPage2".into(), temp.clone()) );
                                   });

                               ui.set_min_height(ui.available_size().y);
                            });

                        // content
                        CentralPanel::default()
                            .show_inside(ui, |ui| {
                               self.meh_renderer.display(ui);
                            });
                    });
            });
    }
}


#[derive(PartialOrd, PartialEq, Clone, Debug, Serialize, Deserialize)]
enum VariablePage { Shader, Camera, Settings, Stats }
fn variable_areas(ctx: &Context, ui: &mut Ui, current: &mut VariablePage) {
   ComboBox::from_label("")
       .selected_text(format!("{:?}", current))
       .show_ui(ui, |ui| {
          ui.selectable_value(current, VariablePage::Shader, format!("{:?}", VariablePage::Shader));
          ui.selectable_value(current, VariablePage::Camera, format!("{:?}", VariablePage::Camera));
          ui.selectable_value(current, VariablePage::Settings, format!("{:?}", VariablePage::Settings));
          ui.selectable_value(current, VariablePage::Stats, format!("{:?}", VariablePage::Stats));
       });

   ui.add_space(15.0);

   ScrollArea::vertical()
      .show(ui, |ui| {

      match current {

         VariablePage::Shader => {}

         VariablePage::Camera => {}

         VariablePage::Settings => {
            ui.group(|ui| {
                  #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
                  enum ThemeMethod { Egui(bool), Catppuccin(CatppuccinThemeWrapper) }

                  #[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
                  pub enum CatppuccinThemeWrapper {
                     Latte,
                     Frappe,
                     Macchiato,
                     Mocha,
                  }
                  impl CatppuccinThemeWrapper {
                     fn cap(&self) -> catppuccin_egui::Theme {
                        match self {
                           CatppuccinThemeWrapper::Latte => catppuccin_egui::LATTE,
                           CatppuccinThemeWrapper::Frappe => catppuccin_egui::FRAPPE,
                           CatppuccinThemeWrapper::Macchiato => catppuccin_egui::MACCHIATO,
                           CatppuccinThemeWrapper::Mocha => catppuccin_egui::MOCHA,
                        }
                     }
                  }

                  let mut current_theme =
                      ctx.memory_mut(|mem| {
                         mem.data.get_persisted("current_theme".into()).unwrap_or_else(|| ThemeMethod::Catppuccin(CatppuccinThemeWrapper::Frappe))
                      });

                  ComboBox::from_label("Select Theme Method")
                      .selected_text(match current_theme {
                         ThemeMethod::Egui(_) => { "Egui" }
                         ThemeMethod::Catppuccin(_) => { "Catppuccin" }
                      })
                      .show_ui(ui, |ui| {
                         ui.selectable_value(&mut current_theme, ThemeMethod::Egui(true), "Egui");
                         ui.selectable_value(&mut current_theme, ThemeMethod::Catppuccin(CatppuccinThemeWrapper::Macchiato), "Catppuccin");
                      });

                  match &mut current_theme {
                     ThemeMethod::Egui(dark_mode) => {
                        ui.checkbox(dark_mode, "Darkmode");
                        ctx.set_visuals(
                           match dark_mode {
                              true => Visuals::dark(),
                              false => Visuals::light(),
                           });
                     }

                     ThemeMethod::Catppuccin(theme) => {
                        ComboBox::from_label("Catppuccin theme")
                            .selected_text(match &theme {
                               CatppuccinThemeWrapper::Latte => "Latte",
                               CatppuccinThemeWrapper::Frappe => "Frappe",
                               CatppuccinThemeWrapper::Macchiato => "Macchiato",
                               CatppuccinThemeWrapper::Mocha => "Mocha",
                            })
                            .show_ui(ui, |ui| {
                               ui.selectable_value(theme, CatppuccinThemeWrapper::Latte, "Latte");
                               ui.selectable_value(theme, CatppuccinThemeWrapper::Frappe, "Frappe");
                               ui.selectable_value(theme, CatppuccinThemeWrapper::Macchiato, "Macchiato");
                               ui.selectable_value(theme, CatppuccinThemeWrapper::Mocha, "Mocha");
                            });

                        catppuccin_egui::set_theme(ctx, theme.cap())
                     }
                  }

                  ctx.memory_mut(|mem| mem.data.insert_persisted("current_theme".into(), current_theme.clone()));
            }); // themes
         }

         VariablePage::Stats => {
            ui.group(|ui| {
                  ui.horizontal(|ui| {
                     ui.heading("Overall fps");
                     ui.label(format!("{}", get!(TIME).fps as i32));
                     {
                        let mut fps_update_rate = ctx.memory_mut(|mem| { mem.data.get_persisted("fps_update_rate".into()).unwrap_or_else(|| 0.65) });
                        let mut fps_graph_amount = ctx.memory_mut(|mem| { mem.data.get_persisted("fps_graph_amount".into()).unwrap_or_else(|| 100) });

                        ui.menu_button("...", |ui| {
                           if ui.button("Clear").clicked() { get!(TIME).past_fps.clear() };
                           ui.add(Slider::new(&mut fps_update_rate, 0.0..=0.99).text("Fps update rate"));
                           ui.add(Slider::new(&mut fps_graph_amount, 10..=250).text("Fps graph amount"));

                           ctx.memory_mut(|mem| mem.data.insert_persisted("fps_update_rate".into(), fps_update_rate.clone()));
                           ctx.memory_mut(|mem| mem.data.insert_persisted("fps_graph_amount".into(), fps_graph_amount.clone()));
                        });

                        get!(TIME).fps_update_interval = fps_update_rate;
                        get!(TIME).fps_amount = fps_graph_amount;
                     } // settings
                  });

                  // ui.set_max_width(200.0); todo limit width

                  let line = Line::new(get!(TIME).past_fps.clone());
                  Plot::new("my_plot")
                      .view_aspect(2.0)
                      .allow_drag(false)
                      .allow_scroll(false)
                      .allow_zoom(false)
                      .allow_boxed_zoom(false)
                      .show_axes(Vec2b::new(false, true))
                      .show(ui, |plot_ui| {
                         plot_ui.line(line);
                      });
               }); // fps

            {
               #[cfg(not(target_arch = "wasm32"))]
               ui.group(|ui| {
                  ui.horizontal(|ui| {
                     ui.heading("System usage");
                  });
               });

               #[cfg(target_arch = "wasm32")]
               ui.group(|ui| {
                  ui.horizontal(|ui| {
                     ui.heading("System usage not available on web");
                  });
               });

            } // system


         }
      }

      ui.set_min_width(ui.available_size().x)
   });
}

#[allow(dead_code)]
fn test_ui(ui: &mut Ui) {
   ScrollArea::vertical()
       .show(ui, |ui| {
          for i in 0..50 {
             if i % 2 == 0 {
                ui.horizontal(|ui| {
                   ui.label("testing the thing and doing the thing");
                   ui.add(DragValue::new(&mut 4))
                });
             }

             else if i % 3 == 0 {
                ui.add(Slider::new(&mut 1002.0, 0.0..=100.0).text(format!("Number {i}")));

                ui.add(Slider::new(&mut 1002.0, 0.0..=100.0).text(format!("Yes {i}")));
             }

             else {
                ui.add(egui::Checkbox::new(&mut false, "and"));
                ui.add(egui::Checkbox::new(&mut false, "then"));
                ui.add(egui::Checkbox::new(&mut false, "the"));
             }
          }
          ui.set_min_width(ui.available_size().x)
       });
}





// #[derive(PartialOrd, PartialEq, Clone, Debug, Serialize, Deserialize)]
// enum DisplayPanel { FPS, Test, }
//
// let mut temp = ctx.memory_mut(|mem| { mem.data.get_persisted("DisplayPanel".into()).unwrap_or_else(|| DisplayPanel::FPS) });
// ui.group(|ui| {
//    ui.horizontal(|ui| {
//       ui.radio_value(&mut temp, DisplayPanel::FPS, "FPS");
//       ui.radio_value(&mut temp, DisplayPanel::Test, "Test");
//       ctx.memory_mut( |mem| mem.data.insert_persisted("DisplayPanel".into(), temp.clone()) );
//    })
// });
//
// match temp {
//    DisplayPanel::FPS => {
//       ui.label("Performance graphs");
//    }
//
//    DisplayPanel::Test => {
//       ui.label("Not Implemented");
//    }
// }