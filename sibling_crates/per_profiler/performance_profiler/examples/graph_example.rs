use std::thread::sleep;
use std::time::Duration;

use eframe::{App, CreationContext, egui, Frame};
use eframe::egui::{CentralPanel, Context, DragValue, Vec2b};
use eframe::egui_wgpu::WgpuConfiguration;
use eframe::wgpu::PresentMode;
use egui_plot::{Corner, Legend, Line, Plot};
use per_macros::time_function;
use performance_code::get_profiler;

fn main() -> eframe::Result {
   let native_options = eframe::NativeOptions {
      vsync: false,
      wgpu_options: WgpuConfiguration {
         present_mode: PresentMode::Immediate,
         ..Default::default()
      },
      viewport: egui::ViewportBuilder::default()
          .with_inner_size([400.0, 300.0])
          .with_min_inner_size([300.0, 220.0]),
      ..Default::default()
   };

   eframe::run_native(
      "Display example",
      native_options,
      Box::new(|cc| Ok(Box::new(DisplayExampleApp::new(cc)))),
   )
}

struct DisplayExampleApp {
   main_delay: u64,
}
impl DisplayExampleApp {
   pub fn new(_cc: &CreationContext) -> Self {
      Self {
         main_delay: 25,
      }
   }
}

impl App for DisplayExampleApp {
   #[time_function("MAIN")]
   fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
      CentralPanel::default()
          .show(ctx, |ui| {
             let lines: Vec<Line> = get_profiler!().profiles.iter().map(|p| {
                Line::new(p.1.timeings.clone())
                    .fill(0.0)
                    .name(
                       format!("{:.2} => {}",
                               p.1.timeings.last().unwrap_or(&[0.0, 0.0])[1],
                               p.0.clone()
                       )
                    )
             }).collect();
             Plot::new("my_plot")
                 .view_aspect(2.0)
                 .allow_drag(false)
                 .allow_scroll(false)
                 .allow_zoom(false)
                 .allow_boxed_zoom(false)
                 .include_y(0.0)
                 .y_axis_label("Milliseconds")
                 .show_axes(Vec2b::new(false, true))
                 .legend(Legend::default().position(Corner::LeftBottom))
                 .show(ui, |plot_ui| {
                    for line in lines {
                       plot_ui.line(line);
                    }
                 });

             ui.add(DragValue::new(&mut self.main_delay).prefix("Test main delay").speed(1.0));
          });


      sleep(Duration::from_millis(self.main_delay));

      self.test_layer_2();

      get_profiler!().resolve_profiler();
      ctx.request_repaint();
   }
}


/// test inner functions
impl DisplayExampleApp {
   #[time_function("LAYER_2")]
   fn test_layer_2(&self) {
      sleep(Duration::from_millis(24));

      self.test_layer_3();
   }

   #[time_function("LAYER_3")]
   fn test_layer_3(&self) {
      sleep(Duration::from_millis(12));
   }
}
