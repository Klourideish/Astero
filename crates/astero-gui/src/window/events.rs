//! Astero-owned Winit event loop, window lifecycle, input and repaint scheduling.
use crate::{model::SessionView, renderer::vulkan::Graphics, ui::Toolkit};
use astero_core::session::SessionObserver;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// Field order matters: destroy Vulkan objects before toolkit and before the native window.
struct LiveWindow {
    graphics: Graphics,
    toolkit: Toolkit,
    window: Window,
    last_frame: Instant,
    occluded: bool,
}
struct Application {
    view: SessionView,
    live: Option<LiveWindow>,
    error: Option<String>,
}

impl Application {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl ToString) {
        self.error = Some(error.to_string());
        event_loop.exit();
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let create = || -> Result<LiveWindow, String> {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Astero - Session Inspector")
                        .with_inner_size(LogicalSize::new(1100.0, 720.0)),
                )
                .map_err(|e| e.to_string())?;
            let mut toolkit = Toolkit::new(&window);
            let graphics = Graphics::new(&window, &mut toolkit.context)
                .map_err(|e| format!("GUI Vulkan initialization: {e}"))?;
            Ok(LiveWindow {
                graphics,
                toolkit,
                window,
                last_frame: Instant::now(),
                occluded: false,
            })
        };
        match create() {
            Ok(live) => self.live = Some(live),
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.live.take();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(live) = &mut self.live else {
            return;
        };
        if live.window.id() != window_id {
            return;
        }
        let wrapped: Event<()> = Event::WindowEvent { window_id, event };
        live.toolkit
            .platform
            .handle_event(live.toolkit.context.io_mut(), &live.window, &wrapped);
        if let Event::WindowEvent { event, .. } = wrapped {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    live.graphics.resize();
                    live.window.request_redraw();
                }
                WindowEvent::Occluded(value) => {
                    live.occluded = value;
                }
                WindowEvent::RedrawRequested if !live.occluded => {
                    let mut render = || -> Result<(), String> {
                        if !live
                            .graphics
                            .prepare(&live.window, &mut live.toolkit.context)
                            .map_err(|e| e.to_string())?
                        {
                            return Ok(());
                        }
                        let now = Instant::now();
                        live.toolkit
                            .context
                            .io_mut()
                            .update_delta_time(now.duration_since(live.last_frame));
                        live.last_frame = now;
                        live.toolkit.prepare(&live.window, &self.view)?;
                        live.graphics
                            .draw(live.toolkit.context.render())
                            .map_err(|e| e.to_string())
                    };
                    if let Err(error) = render() {
                        self.fail(event_loop, format!("GUI render: {error}"));
                    }
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(live) = &self.live {
            let size = live.window.inner_size();
            if !live.occluded && size.width > 0 && size.height > 0 {
                let deadline = live.last_frame + Duration::from_millis(33);
                if Instant::now() >= deadline {
                    live.window.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
                return;
            }
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.live.take();
    }
}

pub fn run(observer: SessionObserver) -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let mut application = Application {
        view: SessionView::new(observer),
        live: None,
        error: None,
    };
    event_loop
        .run_app(&mut application)
        .map_err(|e| e.to_string())?;
    application.error.map_or(Ok(()), Err)
}
