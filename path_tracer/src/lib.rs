use egui::Ui;
use egui_wgpu::RenderState;

pub struct PathTracer {}
impl PathTracer {
   pub fn new(_render_state: &RenderState) -> Self {
      Self {}
   }


   pub fn update(&mut self, render_state: &RenderState) {
      self.render_pass(render_state);
   }


   pub fn display(&mut self, ui: &mut Ui) {
      self.handle_input(ui);
   }

   fn handle_input(&mut self, _ui: &mut Ui) {}

   fn render_pass(&mut self, _render_state: &RenderState) {}
}