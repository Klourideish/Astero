//! Optional host output application. No ImGui context, guest execution or GPU service.
use crate::renderer::vulkan::Graphics;
use astero_video::presentation::{Endpoint, Frame, PresentationSink, synthetic};
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
struct Live {
    graphics: Graphics,
    window: Window,
    last: Option<Frame>,
    unacknowledged: bool,
}
struct Application {
    endpoint: Endpoint,
    live: Option<Live>,
    next: Instant,
    end: Instant,
    id: u64,
    error: Option<String>,
}
impl Application {
    fn fail(&mut self, e: impl ToString, el: &ActiveEventLoop) {
        self.endpoint.failed();
        self.error = Some(e.to_string());
        el.exit();
    }
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let result = (|| {
            let window = el
                .create_window(
                    Window::default_attributes()
                        .with_title("Astero - SYNTHETIC HOST PRESENTATION")
                        .with_inner_size(LogicalSize::new(640., 360.)),
                )
                .map_err(|e| e.to_string())?;
            let graphics = Graphics::host(&window).map_err(|e| e.to_string())?;
            Ok::<_, String>(Live {
                graphics,
                window,
                last: None,
                unacknowledged: false,
            })
        })();
        match result {
            Ok(l) => {
                self.live = Some(l);
                eprintln!("Synthetic host presenter READY; no guest VideoOut/GPU");
            }
            Err(e) => self.fail(e, el),
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(live) = &mut self.live else { return };
        if id != live.window.id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                eprintln!("Synthetic close requested");
                el.exit();
            }
            WindowEvent::Resized(size) => {
                live.graphics.resize();
                live.window.request_redraw();
                eprintln!("Synthetic resize {}x{}", size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                let result = (|| {
                    if !live
                        .graphics
                        .prepare_host(&live.window)
                        .map_err(|e| e.to_string())?
                    {
                        return Ok(());
                    }
                    let incoming = self.endpoint.take();
                    if incoming.is_some() {
                        live.unacknowledged = true;
                    }
                    if let Some(f) = incoming {
                        live.last = Some(f);
                    }
                    if let Some(f) = &live.last
                        && live.graphics.draw_host(f).map_err(|e| e.to_string())?
                        && live.unacknowledged
                    {
                        self.endpoint.completed(f);
                        live.unacknowledged = false;
                        eprintln!("Synthetic frame {} presented", f.id);
                    }
                    Ok::<_, String>(())
                })();
                if let Err(e) = result {
                    self.fail(e, el);
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        let now = Instant::now();
        if now >= self.end {
            el.exit();
            return;
        }
        if now >= self.next {
            self.id += 1;
            if let Err(e) = self.endpoint.present(synthetic(self.id)) {
                self.fail(format!("Frame: {e:?}"), el);
                return;
            }
            if let Some(l) = &self.live {
                l.window.request_redraw();
            }
            self.next = now + Duration::from_millis(500);
        }
        el.set_control_flow(ControlFlow::WaitUntil(self.next.min(self.end)));
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        if let Some(live) = &self.live
            && live.unacknowledged
            && let Some(f) = &live.last
        {
            self.endpoint.discard(f);
        }
        self.live.take();
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.live.take();
        self.endpoint.close();
        eprintln!(
            "Synthetic presenter teardown: {:?}",
            self.endpoint.snapshot()
        );
    }
}
pub fn run(endpoint: Endpoint) -> Result<(), String> {
    let el = EventLoop::new().map_err(|e| e.to_string())?;
    let now = Instant::now();
    let mut app = Application {
        endpoint,
        live: None,
        next: now,
        end: now + Duration::from_secs(60),
        id: 0,
        error: None,
    };
    el.run_app(&mut app).map_err(|e| e.to_string())?;
    app.error.map_or(Ok(()), Err)
}
