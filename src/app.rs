use std::iter;
use std::sync::{Mutex};
use eframe::{App, CreationContext, Frame};
use eframe::egui_wgpu::Renderer;
use eframe::wgpu::{CommandEncoderDescriptor, Device, Extent3d, Queue, TextureFormat};
use egui::load::SizedTexture;
use egui::{Image, Ui, Vec2, Window};
use once_cell::sync::Lazy;
use crate::{get, init_static};
use crate::packages::time_package::TimePackage;
use crate::render_state::meh_renderer::MehRenderer;
use crate::render_state::structs::EguiTexturePackage;
use crate::render_state::test::test_render_pipeline::TestRenderPipeline;


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

    fn update(&mut self) {
        get!(TIME).update();
    }

    fn render(&mut self, frame: &mut Frame) {
        let r_thing = frame.wgpu_render_state().unwrap();
        let device = &r_thing.device;
        let queue = &r_thing.queue;

        self.meh_renderer.render_pass(device, queue)

    }

    fn ui(&mut self, ctx: &egui::Context) {
        Window::new("test")
            .show(ctx, |ui| {
                self.meh_renderer.display(ui);
                ui.label(format!("{}", get!(TIME).fps));
            });
    }
}

impl App for MehApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        self.update();
        self.render(frame);
        self.ui(ctx);

        ctx.request_repaint();
    }
}
