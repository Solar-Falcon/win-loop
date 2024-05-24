use crate::{App, AppInitFunc, Context};
use web_time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{Event, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

struct CreationInfo<A> {
    fps: u32,
    max_frame_time: Duration,
    app_init: Box<AppInitFunc<A>>,
}

pub struct AppHandler<A: App> {
    app_ctx: Option<(A, Context)>,
    creation_info: Option<CreationInfo<A>>,
    instant: Instant,
    accumulated_time: Duration,
}

impl<A: App> AppHandler<A> {
    #[inline]
    pub fn new(app_init: Box<AppInitFunc<A>>, fps: u32, max_frame_time: Duration) -> Self {
        Self {
            app_ctx: None,
            instant: Instant::now(),
            accumulated_time: Duration::ZERO,
            creation_info: Some(CreationInfo {
                app_init,
                fps,
                max_frame_time,
            }),
        }
    }

    #[inline]
    fn pass_event(&mut self, event: Event<()>, event_loop: &ActiveEventLoop) -> Result<(), ()> {
        if let Some((app, _)) = &mut self.app_ctx {
            handle_error(app.handle(event), event_loop)
        } else {
            Ok(())
        }
    }
}

impl<A: App> ApplicationHandler for AppHandler<A> {
    #[inline]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(info) = self.creation_info.take() {
            let app = match (info.app_init)(event_loop) {
                Ok(app) => app,
                Err(err) => {
                    log::error!("{err}");
                    event_loop.exit();
                    return;
                }
            };

            let ctx = Context::new(info.fps, info.max_frame_time);

            self.app_ctx = Some((app, ctx));
        }

        let _ = self.pass_event(Event::Resumed, event_loop);
    }

    #[inline]
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = self.pass_event(Event::Suspended, event_loop);
    }

    #[inline]
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = self.pass_event(Event::LoopExiting, event_loop);
    }

    #[inline]
    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = self.pass_event(Event::MemoryWarning, event_loop);
    }

    #[inline]
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = self.pass_event(Event::NewEvents(cause), event_loop);
    }

    #[inline]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = self.pass_event(Event::UserEvent(event), event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some((app, ctx)) = &mut self.app_ctx {
            ctx.input.process_event(&event);

            if event == WindowEvent::CloseRequested {
                event_loop.exit();
            }

            let _ = handle_error(
                app.handle(Event::WindowEvent { window_id, event }),
                event_loop,
            );
        }
    }

    #[inline]
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = self.pass_event(Event::DeviceEvent { device_id, event }, event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some((app, ctx)) = &mut self.app_ctx {
            if handle_error(app.handle(Event::AboutToWait), event_loop).is_err() {
                return;
            }

            let mut elapsed = self.instant.elapsed();
            self.instant = Instant::now();

            if elapsed > ctx.max_frame_time {
                elapsed = ctx.max_frame_time;
            }

            self.accumulated_time += elapsed;

            let mut keys_updated = false;

            while self.accumulated_time > ctx.target_frame_time {
                ctx.delta_time = ctx.target_frame_time;

                if handle_error(app.update(ctx), event_loop).is_err() {
                    return;
                }

                if ctx.exit {
                    event_loop.exit();
                    return;
                }

                if !keys_updated {
                    ctx.input.update_keys();
                    keys_updated = true;
                }

                self.accumulated_time = self.accumulated_time.saturating_sub(ctx.target_frame_time);

                let blending_factor =
                    self.accumulated_time.as_secs_f64() / ctx.target_frame_time.as_secs_f64();

                let _ = handle_error(app.render(blending_factor), event_loop);
            }
        }
    }
}

#[inline]
fn handle_error(result: anyhow::Result<()>, el: &ActiveEventLoop) -> Result<(), ()> {
    if let Err(error) = result {
        log::error!("{}", error);
        el.exit();

        Err(())
    } else {
        Ok(())
    }
}
