use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::renderer::Renderer;

pub struct App {
    pub window: Option<Arc<Window>>,
    #[allow(dead_code)] // TODO: Remove this once the renderer is implemented.
    pub renderer: Option<Renderer>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
        }
    }

    fn handle_key_event(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}

// Boilerplate code was sourced from the winit documentation.
// https://docs.rs/winit/latest/winit
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Phantom"))
                .unwrap(),
        ));

        // Use block_on to handle the async renderer creation in a sync context
        let renderer =
            futures::executor::block_on(Renderer::new(self.window.as_ref().unwrap().clone()));

        match renderer {
            Ok(renderer) => self.renderer = Some(renderer),
            Err(e) => {
                eprintln!("Failed to create renderer: {}", e);
                // You might want to exit the application here or handle the error differently
                return;
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // TODO: Update state as needed
                match self.renderer.as_mut().unwrap().render() {
                    Ok(_) => (),
                    // Reconfig surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = self.window.as_ref().unwrap().inner_size();
                        self.renderer
                            .as_mut()
                            .unwrap()
                            .resize(size.width, size.height);
                    }
                    Err(e) => eprintln!("Error rendering: {}", e),
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                self.renderer
                    .as_mut()
                    .unwrap()
                    .resize(physical_size.width, physical_size.height);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key_event(event_loop, code, key_state.is_pressed()),
            _ => (),
        }
    }
}
