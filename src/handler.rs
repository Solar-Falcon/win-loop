use crate::{App, Context};
use std::sync::Arc;
use web_time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{Event, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

// a needlessly complicated construct to avoid cloning WindowAttributes
#[derive(Debug)]
enum AppCtx {
    Ctx(Context),
    Info(CtxCreationInfo),
    TemporarilyEmpty,
}

impl AppCtx {
    #[inline]
    fn get_ctx(&mut self) -> Option<&mut Context> {
        match self {
            AppCtx::Ctx(ctx) => Some(ctx),
            _ => None,
        }
    }

    fn construct_ctx(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if matches!(self, AppCtx::Info(_)) {
            let ac = std::mem::replace(self, AppCtx::TemporarilyEmpty);

            let info = match ac {
                AppCtx::Info(info) => info,
                _ => unreachable!(),
            };

            let window = event_loop.create_window(info.attrs)?;

            *self = AppCtx::Ctx(Context::new(
                Arc::new(window),
                info.fps,
                info.max_frame_time,
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
struct CtxCreationInfo {
    attrs: WindowAttributes,
    fps: u32,
    max_frame_time: Duration,
}

#[derive(Debug)]
pub struct AppHandler<A: App> {
    context: AppCtx,
    instant: Instant,
    app: A,
    accumulated_time: Duration,
}

impl<A: App> AppHandler<A> {
    #[inline]
    pub fn new(attrs: WindowAttributes, fps: u32, max_frame_time: Duration, app: A) -> Self {
        Self {
            context: AppCtx::Info(CtxCreationInfo {
                attrs,
                fps,
                max_frame_time,
            }),
            app,
            instant: Instant::now(),
            accumulated_time: Duration::ZERO,
        }
    }
}

impl<A: App> ApplicationHandler for AppHandler<A> {
    #[inline]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if handle_error(self.context.construct_ctx(event_loop), event_loop).is_err() {
            return;
        }

        let _ = handle_error(self.app.handle(Event::Resumed), event_loop);
    }

    #[inline]
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = handle_error(self.app.handle(Event::Suspended), event_loop);
    }

    #[inline]
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = handle_error(self.app.handle(Event::LoopExiting), event_loop);
    }

    #[inline]
    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = handle_error(self.app.handle(Event::MemoryWarning), event_loop);
    }

    #[inline]
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = handle_error(self.app.handle(Event::NewEvents(cause)), event_loop);
    }

    #[inline]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = handle_error(self.app.handle(Event::UserEvent(event)), event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ctx) = self.context.get_ctx() {
            if ctx.window.id() == window_id {
                ctx.input.process_event(&event);

                if event == WindowEvent::CloseRequested {
                    event_loop.exit();
                }
            }
        }

        let _ = handle_error(
            self.app.handle(Event::WindowEvent { window_id, event }),
            event_loop,
        );
    }

    #[inline]
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = handle_error(
            self.app.handle(Event::DeviceEvent { device_id, event }),
            event_loop,
        );
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if handle_error(self.app.handle(Event::AboutToWait), event_loop).is_err() {
            return;
        }

        if let Some(ctx) = self.context.get_ctx() {
            let mut elapsed = self.instant.elapsed();
            self.instant = Instant::now();

            if elapsed > ctx.max_frame_time {
                elapsed = ctx.max_frame_time;
            }

            self.accumulated_time += elapsed;

            let mut keys_updated = false;

            while self.accumulated_time > ctx.target_frame_time {
                ctx.delta_time = ctx.target_frame_time;

                if handle_error(self.app.update(ctx), event_loop).is_err() {
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

                let _ = handle_error(self.app.render(blending_factor), event_loop);
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
