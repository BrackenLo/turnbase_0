//====================================================================

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::StartCause,
    event_loop::{ActiveEventLoop, EventLoop},
};

use super::{tools::Size, State};

//====================================================================

#[derive(Clone)]
pub struct Window(pub Arc<winit::window::Window>);
impl Window {
    #[inline]
    pub fn size(&self) -> Size<u32> {
        self.0.inner_size().into()
    }
}

//====================================================================

pub struct Runner {
    state: Option<State>,
}

impl Runner {
    pub fn run() {
        EventLoop::new()
            .unwrap()
            .run_app(&mut Self { state: None })
            .unwrap();
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::trace!("App Resumed - Creating state.");

        match self.state {
            Some(_) => log::warn!("State already exists."),
            None => self.state = Some(State::new(event_loop)),
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
