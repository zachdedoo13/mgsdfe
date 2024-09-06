use std::sync::Mutex;
use std::time::Duration;

use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, ComboBox, Context, DragValue, ScrollArea, SidePanel, Slider, TopBottomPanel, Ui, Vec2b, Visuals};
use egui::panel::{Side, TopBottomSide};
use egui_plot::{Line, Plot};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use wgpu::AdapterInfo;

use graphing_stuff::graph::NodeGraph;
use rendering_stuff::render_state::MehRenderer;
use rendering_stuff::utility::structs::{RenderPack, RenderSettings};

use crate::{get, init_static, load_persisted, load_temp, render_pack_from_frame, save_persisted, save_temp};
use crate::packages::time_package::TimePackage;
use crate::utility::functions::oss;

// Globals
init_static!(TIME: TimePackage => {TimePackage::new()});

init_static!(RENDER_SETTINGS: RenderSettings => {RenderSettings::default()});


pub struct MehApp {
   meh_renderer: MehRenderer,
   node_graph: NodeGraph,
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
      let render_state = cc.wgpu_render_state.as_ref().expect("wgpu_render_state is None");
      let renderer = &mut render_state.renderer.write();
      let device = &render_state.device;
      let queue = &render_state.queue;

      let meh_renderer = MehRenderer::new(device, queue, renderer, &get!(RENDER_SETTINGS));

      // graph
      let node_graph = NodeGraph::new(cc);

      // init
      let ctx = &cc.egui_ctx;
      set_theme(&cc.egui_ctx);
      {
         let selected_aspect = load_persisted!(ctx, "selected_aspect", {(16, 9)} );
         let aspect_scale = load_persisted!(ctx, "aspect_scale", 1000);

         let aspect = selected_aspect.1 as f32 / selected_aspect.0 as f32;

         let h = (aspect_scale as f32 * aspect) as u32;
         let w = (aspect_scale as f32) as u32;

         get!(RENDER_SETTINGS).width = w;
         get!(RENDER_SETTINGS).height = h;
      } // init aspect of shader
      {
         let rs = &mut load_persisted!(ctx, "render_settings", RenderSettings::default());
         rs.path_tracer_uniform_settings.cam_dir = [0.0, 0.0, 0.0];

         get!(RENDER_SETTINGS) = *rs;
      }


      // init temp values
      let adapter_info = render_state.adapter.get_info();
      cc.egui_ctx.memory_mut(|mem| mem.data.insert_temp("adapter_info".into(), adapter_info));


      Self {
         meh_renderer,
         node_graph,
      }
   }

   fn update(&mut self, render_pack: &mut RenderPack<'_>) {
      get!(TIME).update();

      let render_settings = &mut get!(RENDER_SETTINGS);
      render_settings.path_tracer_uniform_settings.time = get!(TIME).start_time.elapsed().as_secs_f32();

      self.meh_renderer.update(render_pack, render_settings)
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
                 .max_width(ctx.screen_rect().max.x * 0.7)
                 .show_inside(ui, |ui| {
                    // sdf editor

                    // ui.group(|ui| { ui.label("big\nbad\nsdf\neditor"); });
                    self.node_graph.update(ui);

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
                                  ctx.memory_mut(|mem| mem.data.insert_persisted("VarPage1".into(), temp.clone()));

                                  ui.set_min_width(ui.available_size().x);
                               });

                           // stats
                           CentralPanel::default()
                               .show_inside(ui, |ui| {
                                  let mut temp = ctx.memory_mut(|mem| { mem.data.get_persisted("VarPage2".into()).unwrap_or_else(|| VariablePage::Stats) });
                                  variable_areas(ctx, ui, &mut temp);
                                  ctx.memory_mut(|mem| mem.data.insert_persisted("VarPage2".into(), temp.clone()));
                               });

                           ui.set_min_height(ui.available_size().y);
                        });

                    // content
                    CentralPanel::default()
                        .show_inside(ui, |ui| {
                           self.meh_renderer.display(ui, &mut get!(RENDER_SETTINGS));
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
             VariablePage::Shader => {
                shader_settings(ctx, ui);
             }

             VariablePage::Camera => {}

             VariablePage::Settings => {
                settings(ctx, ui);
             }

             VariablePage::Stats => {
                stats(ctx, ui);
             }
          }

          ui.set_min_width(ui.available_size().x)
       });
}


fn stats(ctx: &Context, ui: &mut Ui) {
   {
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
   } // fps

   {
      let info = ctx.memory(|mem|
      match mem.data.get_temp::<AdapterInfo>("adapter_info".into()) {
         Some(adapter_info) => {
            format!(
               "Adapter Info: \n\tName -> {}\n\tVendor -> {}\n\tDevice -> {}\n\tDevice Type -> {:?}\n\tDriver -> {}\n\tDriver Info -> {}\n\tBackend -> {:?}",
               adapter_info.name,
               adapter_info.vendor,
               adapter_info.device,
               adapter_info.device_type,
               adapter_info.driver,
               adapter_info.driver_info,
               adapter_info.backend
            )
         }
         None => "None Found".to_string(),
      }
      );

      ui.group(|ui| {
         ui.label(info);
      });
   } // adapter info

   {
      #[cfg(not(target_arch = "wasm32"))]
      ui.group(|ui| {
         ui.horizontal(|ui| {
            ui.heading("Haven't got around to system usage yet");
         });
      });

      #[cfg(target_arch = "wasm32")]
      ui.group(|ui| {
         ui.horizontal(|ui| {
            ui.heading("System usage not available on web");
         });
      });
   } // system info

   {
      let samples = 200;

      let mut freq = load_temp!(ctx, "freq", 1.0);
      let mut amp = load_temp!(ctx, "amp", 1.0);
      let mut phase = load_temp!(ctx, "phase", 1.0);

      let mut data: Vec<[f64; 2]> = vec![];
      for i in 0..samples {
         let rf = i as f64 / samples as f64;

         data.push([i as f64, oss(rf, freq, amp, phase)])
      }

      ui.add(DragValue::new(&mut freq).prefix("freq").speed(0.05));
      ui.add(DragValue::new(&mut amp).prefix("amp").speed(0.05));
      ui.add(DragValue::new(&mut phase).prefix("phase").speed(0.05));

      let line = Line::new(data);
      Plot::new("my_plot")
          .view_aspect(2.0)
          .allow_drag(false)
          .allow_scroll(false)
          .allow_zoom(false)
          .allow_boxed_zoom(false)
          .show_axes(Vec2b::new(false, true))
          .show(ui, |plot_ui| plot_ui.line(line));


      save_temp!(ctx, "freq", freq);
      save_temp!(ctx, "amp", amp);
      save_temp!(ctx, "phase", phase);
   } // test
}


#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
enum ThemeMethod { Egui(bool), Catppuccin(CatppuccinThemeWrapper) }

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum CatppuccinThemeWrapper {
   Latte,
   Frappe,
   Macchiato,
   Mocha,
}

fn settings(ctx: &Context, ui: &mut Ui) {
   ui.group(|ui| {
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

      ComboBox::from_label("Theme Method")
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
            ComboBox::from_label("Theme")
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

   ui.group(|ui| {
      let mut zoom = ctx.zoom_factor();

      ui.label("Ui Zoom Factor");
      ui.horizontal(|ui| {
         ui.radio_value(&mut zoom, 0.25, "0.25");
         ui.radio_value(&mut zoom, 0.5, "0.5");
         ui.radio_value(&mut zoom, 0.75, "0.75");
      });

      ui.horizontal(|ui| {
         ui.radio_value(&mut zoom, 1.0, "1.0");
         ui.radio_value(&mut zoom, 1.25, "1.25");
         ui.radio_value(&mut zoom, 1.5, "1.5");
      });

      ui.horizontal(|ui| {
         ui.radio_value(&mut zoom, 1.75, "1.75");
         ui.radio_value(&mut zoom, 2.0, "2.0");
         ui.radio_value(&mut zoom, 2.25, "2.25");
      });

      ctx.set_zoom_factor(zoom)
   });
}

fn set_theme(ctx: &Context) {
   let mut current_theme = ctx.memory_mut(|mem| {
      mem.data.get_persisted("current_theme".into()).unwrap_or_else(|| ThemeMethod::Catppuccin(CatppuccinThemeWrapper::Frappe))
   });

   match &mut current_theme {
      ThemeMethod::Egui(dark_mode) => {
         ctx.set_visuals(
            match dark_mode {
               true => Visuals::dark(),
               false => Visuals::light(),
            });
      }

      ThemeMethod::Catppuccin(theme) => {
         catppuccin_egui::set_theme(ctx, theme.cap())
      }
   }
} // for initialisation


fn shader_settings(ctx: &Context, ui: &mut Ui) {
   let rs = &mut get!(RENDER_SETTINGS);

   ui.group(|ui| {
      let mut maintain_aspect_ratio = load_persisted!(ctx, "maintain_aspect_ratio", true);

      ui.horizontal(|ui| {
         ui.label("Resolution settings");
      });

      ui.group(|ui| {
         ui.horizontal(|ui| {
            toggle_ui_compact(ui, &mut maintain_aspect_ratio);
            ui.label("Maintain aspect");
            rs.maintain_aspect_ratio = maintain_aspect_ratio;
         });
      });

      {
         let mut selected_aspect = load_persisted!(ctx, "selected_aspect", {(16, 9)} );
         let mut aspect_scale = load_persisted!(ctx, "aspect_scale", 1000);

         let aspect_ratios = vec![
            (16, 9),
            (4, 3),
            (21, 9),
            (1, 1),
         ];

         ComboBox::from_label("Aspect Ratio")
             .selected_text(format!("{}:{}", selected_aspect.0, selected_aspect.1))
             .show_ui(ui, |ui| {
                for aspect in &aspect_ratios {
                   ui.selectable_value(&mut selected_aspect, *aspect, format!("{}:{}", aspect.0, aspect.1));
                }
             });

         let aspect = selected_aspect.1 as f32 / selected_aspect.0 as f32;
         ui.add(Slider::new(&mut aspect_scale, 8..=3840).text("Scale"));

         let h = (aspect_scale as f32 * aspect) as u32;
         let w = (aspect_scale as f32) as u32;

         ui.label(format!("Height -> {h}  |  Width -> {w}"));
         ui.label(format!("Total pixels => {}", w * h));

         rs.width = w;
         rs.height = h;


         save_persisted!(ctx, "selected_aspect", selected_aspect);
         save_persisted!(ctx, "aspect_scale", aspect_scale);
      }

      save_persisted!(ctx, "maintain_aspect_ratio", maintain_aspect_ratio);
   }); // image size
   ui.group(|ui| {
      ui.label("position");
      ui.horizontal(|ui| {
         ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.cam_pos[0]).speed(0.01));
         ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.cam_pos[1]).speed(0.01));
         ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.cam_pos[2]).speed(0.01));
      });
   }); // pos
   ui.group(|ui| {
      ui.label("raymarch settings");
      ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.steps_per_ray));

      ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.mode).speed(0.1).prefix("start_eps"));
      ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.max_dist).speed(0.001).prefix("max_dist"));
      ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.relaxation).speed(0.001).prefix("relaxation"));
      ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.step_scale_factor).speed(0.001).prefix("step_scale_factor"));
      ui.add(DragValue::new(&mut rs.path_tracer_uniform_settings.eps_scale).speed(0.001).prefix("eps_scale"));
   }); // raymarch

   if ui.button("remake pipeline").clicked() {
      rs.remake_pipeline = true;
   }

   save_persisted!(ctx, "render_settings", *rs);
}


// my fgb
fn toggle_ui_compact(ui: &mut Ui, on: &mut bool) -> egui::Response {
   let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
   let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
   if response.clicked() {
      *on = !*on;
      response.mark_changed();
   }
   response.widget_info(|| {
      egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
   });

   if ui.is_rect_visible(rect) {
      let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
      let visuals = ui.style().interact_selectable(&response, *on);
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