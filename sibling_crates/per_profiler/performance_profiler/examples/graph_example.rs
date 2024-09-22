
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
      let mut profiler = performance_profiler::PROFILER.lock().unwrap();

      // get_profiler!().active = true;

      CentralPanel::default()
          .show(ctx, |ui| {
             let lines: Vec<Line> = profiler.profiles.iter().map(|p| {
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

             ui.add(DragValue::new(&mut self.main_delay).prefix("Test delay").speed(1.0));
          });


      sleep(Duration::from_millis(self.main_delay));

      // self.test(); // dose the weird halt when called, mutex lock indication, test this gpt code

//       extern crate proc_macro;
// 
//       use proc_macro::TokenStream;
//       use quote::quote;
//       use std::cell::RefCell;
//       use syn::{parse_macro_input, ItemFn, LitStr};
// 
//       // Thread-local variable to track if the lock is already held by the current thread
//       thread_local! {
//     static LOCK_HELD: RefCell<bool> = RefCell::new(false);
// }
// 
//       #[proc_macro_attribute]
//       pub fn time_function(attr: TokenStream, input: TokenStream) -> TokenStream {
//          // Parse the input tokens into a syntax tree
//          let input = parse_macro_input!(input as ItemFn);
//          let name = parse_macro_input!(attr as LitStr).value();
// 
//          // Get the function's identifier, signature, and block
//          let fn_sig = &input.sig;
//          let fn_block = &input.block;
// 
//          // Generate the new function body with the placeholders
//          let expanded = quote! {
//         #fn_sig {
//             if !LOCK_HELD.with(|lock_held| *lock_held.borrow()) {
//                 LOCK_HELD.with(|lock_held| *lock_held.borrow_mut() = true);
//                 performance_code::PROFILER.lock().unwrap().start_time_function(#name);
//                 #fn_block
//                 performance_code::PROFILER.lock().unwrap().end_time_function(#name);
//                 LOCK_HELD.with(|lock_held| *lock_held.borrow_mut() = false);
//             } else {
//                 #fn_block
//             }
//         }
//     };

         // Return the generated tokens
      //    TokenStream::from(expanded)
      // }

      profiler.resolve_profiler();
      ctx.request_repaint();

      // profiler.end_time_function("MAIN");
   }
}

impl DisplayExampleApp {
   #[time_function("TEST_SLEEP")]
   fn test(&self) {
      sleep(Duration::from_millis(self.main_delay));
   }
}
