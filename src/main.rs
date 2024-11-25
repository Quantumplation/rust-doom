pub mod graphics;
pub mod renderer;

use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result};
use cgmath::{prelude::*, Basis2, Rad, Vector2};
use graphics::Graphics;
use renderer::Camera;
use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

struct State<'a> {
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,

    graphics: graphics::Graphics<'a>,
    camera: Rc<RefCell<Camera>>,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> Result<State<'a>> {
        let size = window.inner_size();
        let camera = Rc::new(RefCell::new(Camera {
            player_pos: Vector2::new(5., 5.),
            facing_dir: Vector2::new(-1., 0.1),
            view_plane: Vector2::new(0., 0.66),
        }));
        let graphics = Graphics::new(camera.clone(), window, size)
            .await
            .context("failed to construct graphics")?;
        Ok(State {
            size,
            window,
            graphics,
            camera,
        })
    }

    pub fn event_loop(&mut self, event: Event<()>, control_flow: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == self.window.id() {
                    if self.input(&event) {
                        return;
                    }
                    if !self.handle_event(&event) {
                        control_flow.exit();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
            }
            WindowEvent::RedrawRequested => {
                self.window().request_redraw();
                self.update();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.resize(self.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("OutOfMemory");
                        return false;
                    }

                    // This happens when the a frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            _ if is_close_event(event) => return false,
            _ => {}
        }
        true
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.graphics.resize(new_size);
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let mut camera = self.camera.borrow_mut();
        let angle: f32 = 0.007; //0.005f32;
        camera.facing_dir = Vector2::new(
            camera.facing_dir.x * angle.cos() - camera.facing_dir.y * angle.sin(),
            camera.facing_dir.x * angle.sin() + camera.facing_dir.y * angle.cos(),
        );
        camera.view_plane = Vector2::new(
            camera.view_plane.x * angle.cos() - camera.view_plane.y * angle.sin(),
            camera.view_plane.x * angle.sin() + camera.view_plane.y * angle.cos(),
        );
    }

    fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        self.graphics.render()
    }
}

fn is_close_event(event: &WindowEvent) -> bool {
    return match event {
        WindowEvent::CloseRequested => true,
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
            ..
        } => true,
        _ => false,
    };
}

async fn run() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new().context("failed to construct event loop")?;
    let window = WindowBuilder::new()
        .with_title("Rust Doom")
        .build(&event_loop)
        .context("failed to construct window")?;

    let mut state = State::new(&window)
        .await
        .context("failed to construct state")?;

    event_loop
        .run(move |event, control_flow| state.event_loop(event, control_flow))
        .context("failed to run event loop")
}

fn main() -> Result<()> {
    pollster::block_on(run())
}
