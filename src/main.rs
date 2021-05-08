#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]

use crate::gui::Gui;
use crate::world::World;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use simple_logger::SimpleLogger;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod cpu;
mod gui;
mod world;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 320;

/// Representation of the application state. In this example, a box will bounce around the screen.

fn main() -> Result<(), Error> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("chip8", log::LevelFilter::Trace)
        .init()
        .unwrap();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + egui")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut gui) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(64, 32, surface_texture)?;
        let gui = Gui::new(
            window_size.width,
            window_size.height,
            scale_factor,
            pixels.context(),
            World::new(),
        );

        (pixels, gui)
    };

    let rom = include_bytes!("./trip8.ch8");
    
    gui.world.cpu.load_rom(rom);

    // gui.world.cpu.load_rom(&[
    //     0xF0, 0x29, // LD F, V0
    //     0x60, 63,
    //     0x61, 31,
    //     0xD0, 0x05, // DRW V0, V0, 5
    //     0x00, 0xEE, // RET #(halt)
    //     ]);

    event_loop.run(move |event, _, control_flow| {
        // Update egui inputs
        gui.handle_event(&event);

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            // Draw the world
            gui.world.draw(pixels.get_frame());

            // Prepare egui
            gui.prepare();

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);

                // Render egui
                gui.render(encoder, render_target, context);
            });

            // Basic error handling
            if render_result
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                gui.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                gui.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
        }

        gui.world.update();
    });
}
