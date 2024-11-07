//====================================================================

use std::{marker::PhantomData, sync::Arc};

use common::Size;
use winit::{
    application::ApplicationHandler,
    event::StartCause,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowAttributes,
};

use crate::scene::Scene;

use super::State;

//====================================================================

#[derive(Clone)]
pub struct Window(pub Arc<winit::window::Window>);
impl Window {
    pub(super) fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys};

            log::info!("Adding canvas to window");

            match window.request_inner_size(PhysicalSize::new(450, 400)) {
                Some(_) => {}
                None => log::warn!("Got none when requesting window inner size"),
            };

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("game")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        Self(Arc::new(window))
    }

    #[inline]
    pub fn size(&self) -> Size<u32> {
        let window_size = self.0.inner_size();

        Size {
            width: window_size.width,
            height: window_size.height,
        }
    }
}

//====================================================================

pub struct Runner<S: Scene> {
    state: Option<State>,
    default_scene: PhantomData<S>,
}

impl<S: Scene> Runner<S> {
    pub fn run() {
        EventLoop::new()
            .unwrap()
            .run_app(&mut Self {
                state: None,
                default_scene: PhantomData,
            })
            .unwrap();
    }
}

impl<S: Scene> ApplicationHandler for Runner<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::trace!("App Resumed - Creating state.");

        match self.state {
            Some(_) => log::warn!("State already exists."),
            None => self.state = Some(State::new::<S>(event_loop)),
        }
    }

    #[inline]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.window_event(event_loop, window_id, event);
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let Some(state) = &mut self.state {
            if let StartCause::ResumeTimeReached { .. } = cause {
                state.request_redraw();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.device_event(event_loop, device_id, event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}

//====================================================================
