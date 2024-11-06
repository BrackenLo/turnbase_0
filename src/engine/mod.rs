//====================================================================

use std::{sync::Arc, time::Duration};

use renderer::Renderer;
use scene::Scene;
use tools::{Input, Time};
use window::Window;
use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{WindowAttributes, WindowId},
};

use crate::game::scenes::EmptyScene;

pub mod renderer;
pub mod scene;
pub mod tools;
pub mod window;

//====================================================================

const DEFAULT_FPS: f32 = 1. / 75.;

pub struct State {
    inner: StateInner,
    scene: Box<dyn Scene>,
}

pub struct StateInner {
    pub fps: Duration,
    pub window: Window,
    pub renderer: Renderer,
    pub keys: Input<KeyCode>,
    pub time: Time,
}

impl State {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let fps = Duration::from_secs_f32(DEFAULT_FPS);
        let window = Window(Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        ));

        let renderer = Renderer::new(&window);

        Self {
            inner: StateInner {
                fps,
                window,
                renderer,
                keys: Input::default(),
                time: Time::default(),
            },
            scene: Box::new(EmptyScene::new()),
        }
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    log::warn!(
                        "Window resized to invalid size ({}, {})",
                        physical_size.width,
                        physical_size.height
                    );
                    return;
                }
                let size = physical_size.into();
                self.inner.renderer.resize(size);
            }

            WindowEvent::CloseRequested => {
                log::info!("Close requested. Closing App");
                event_loop.exit();
            }

            WindowEvent::Destroyed => log::error!("Window was destroyed"),

            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(key) = event.physical_key {
                    tools::process_inputs(&mut self.inner.keys, key, event.state.is_pressed())
                }
            }
            //
            // WindowEvent::CursorMoved { position, .. } => {}
            // WindowEvent::MouseWheel { delta, .. } => {}
            // WindowEvent::MouseInput { state, button, .. } => {}
            //
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::wait_duration(
                    self.inner.fps,
                ));

                self.tick();
            }

            _ => {}
        }
    }

    pub fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.inner.window.0.request_redraw();
    }

    pub fn tick(&mut self) {
        tools::tick_time(&mut self.inner.time);

        self.scene.tick(&mut self.inner);
        self.inner.renderer.tick();

        tools::reset_input(&mut self.inner.keys);
    }
}

//====================================================================
