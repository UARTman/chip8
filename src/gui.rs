use egui::{ClippedMesh, FontDefinitions};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use pixels::{wgpu, PixelsContext};
use std::time::Instant;

use crate::{World, cpu::ExecutionState};

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Gui {
    // State for egui.
    start_time: Instant,
    platform: Platform,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,

    // State for the demo app.
    // window_open: bool,
    pub world: World,
}

impl Gui {
    /// Create egui.
    pub(crate) fn new(
        width: u32,
        height: u32,
        scale_factor: f64,
        context: &PixelsContext,
        world: World,
    ) -> Self {
        let platform = Platform::new(PlatformDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor: scale_factor as f32,
        };
        let rpass = RenderPass::new(&context.device, wgpu::TextureFormat::Bgra8UnormSrgb);

        Self {
            start_time: Instant::now(),
            platform,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            // window_open: true,
            world,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::Event<'_, ()>) {
        self.platform.handle_event(event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.screen_descriptor.physical_width = width;
        self.screen_descriptor.physical_height = height;
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.scale_factor = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self) {
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());

        // Begin the egui frame.
        self.platform.begin_frame();

        // Draw the demo application.
        self.ui(&self.platform.context());

        // End the egui frame and create all paint jobs to prepare for rendering.
        let (_output, paint_commands) = self.platform.end_frame();
        self.paint_jobs = self.platform.context().tessellate(paint_commands);
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &egui::CtxRef) {
        egui::TopPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    ui.set_enabled(self.world.cpu.execution_state == ExecutionState::Paused);
                    if ui.button("Load ROM").clicked() {
                        log::debug!("Opening file dialog...");
                        let file = rfd::FileDialog::new().set_directory(".").pick_file();
                        if let Some(file) = file {
                            log::debug!("File chosen: {}", file.to_string_lossy());
                            if let Ok(mut file) = std::fs::File::open(file) {
                                use std::io::Read;
                                let mut rom = Vec::new();
                                if file.read_to_end(&mut rom).is_ok() {
                                    self.world.cpu.load_rom(&rom)
                                }
                            }
                        }
                    }
                    // if ui.button("About...").clicked() {
                    //     self.window_open = true;
                    // }
                })
            });
        });

        egui::Window::new("CPU controls").show(ctx, |ui| {
            self.world.cpu.draw_ui(ui);
        });
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        self.rpass.update_texture(
            &context.device,
            &context.queue,
            &self.platform.context().texture(),
        );
        self.rpass
            .update_user_textures(&context.device, &context.queue);
        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Record all render passes.
        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        );
    }
}
