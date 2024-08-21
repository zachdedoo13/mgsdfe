use std::sync::{Mutex};
use std::time::Duration;
use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, Context, DragValue, ScrollArea, SidePanel, Slider, TopBottomPanel, Ui};
use egui::panel::{Side, TopBottomSide};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use crate::{get, init_static, render_pack_from_frame};
use crate::packages::time_package::TimePackage;
use crate::render_state::meh_renderer::MehRenderer;
use crate::render_state::structs::RenderPack;


// Globals
init_static!(TIME: TimePackage => {TimePackage::new()});


pub struct MehApp {
    meh_renderer: MehRenderer,
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
                                      #[derive(PartialOrd, PartialEq, Clone, Debug, Serialize, Deserialize)]
                                      enum SettingsPage { Shader, Camera, }

                                      let mut temp = ctx.memory_mut(|mem| { mem.data.get_persisted("SettingsPage".into()).unwrap_or_else(|| SettingsPage::Shader) });
                                      ui.group(|ui| {
                                         ui.horizontal(|ui| {
                                            ui.radio_value(&mut temp, SettingsPage::Shader, "Shader");
                                            ui.radio_value(&mut temp, SettingsPage::Camera, "Camera");
                                            ctx.memory_mut( |mem| mem.data.insert_persisted("SettingsPage".into(), temp.clone()) );
                                         })
                                      });

                                      match temp {
                                         SettingsPage::Shader => {
                                            ui.label("Performance graphs");
                                         }

                                         SettingsPage::Camera => {
                                            ui.label("Not Implemented");
                                         }
                                      }

                                      ui.set_min_width(ui.available_size().x);
                                   });

                               // stats
                               CentralPanel::default()
                                   .show_inside(ui, |ui| {
                                      #[derive(PartialOrd, PartialEq, Clone, Debug, Serialize, Deserialize)]
                                      enum DisplayPanel { FPS, Test, }

                                      let mut temp = ctx.memory_mut(|mem| { mem.data.get_persisted("DisplayPanel".into()).unwrap_or_else(|| DisplayPanel::FPS) });
                                      ui.group(|ui| {
                                         ui.horizontal(|ui| {
                                            ui.radio_value(&mut temp, DisplayPanel::FPS, "FPS");
                                            ui.radio_value(&mut temp, DisplayPanel::Test, "Test");
                                            ctx.memory_mut( |mem| mem.data.insert_persisted("DisplayPanel".into(), temp.clone()) );
                                         })
                                      });

                                      match temp {
                                         DisplayPanel::FPS => {
                                            ui.label("Performance graphs");
                                         }

                                         DisplayPanel::Test => {
                                            ui.label("Not Implemented");
                                         }
                                      }
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

