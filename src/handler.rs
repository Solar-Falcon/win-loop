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
enum AppCtx<A, D> {
    Init {
        app: A,
        ctx: Context,
    },
    Info(CtxCreationInfo<D>),
    TemporarilyEmpty,
}

impl<A, D> AppCtx<A, D> {
    #[inline]
    fn get(&mut self) -> Option<(&mut A, &mut Context)> {
        match self {
            AppCtx::Init { app, ctx } => Some((app, ctx)),
            _ => None,
        }
    }

    fn construct_ctx(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> where A: App<D> {
        if matches!(self, AppCtx::Info(_)) {
            let ac = std::mem::replace(self, AppCtx::TemporarilyEmpty);

            let info = match ac {
                AppCtx::Info(info) => info,
                _ => unreachable!(),
            };

            let window = Arc::new(event_loop.create_window(info.attrs)?);

            *self = AppCtx::Init {
                app: A::init(info.app_creation_data, &window)?,
                ctx: Context::new(
                    window,
                    info.fps,
                    info.max_frame_time,
                ),
            };
        }

        Ok(())
    }
}

#[derive(Debug)]
struct CtxCreationInfo<D> {
    attrs: WindowAttributes,
    fps: u32,
    max_frame_time: Duration,
    app_creation_data: D,
}

#[derive(Debug)]
pub struct AppHandler<A: App<D>, D> {
    context: AppCtx<A, D>,
    instant: Instant,
    accumulated_time: Duration,
}

impl<A: App<D>, D> AppHandler<A, D> {
    #[inline]
    pub fn new(attrs: WindowAttributes, fps: u32, max_frame_time: Duration, app_creation_data: D) -> Self {
        Self {
            context: AppCtx::Info(CtxCreationInfo {
                attrs,
                fps,
                max_frame_time,
                app_creation_data
            }),
            instant: Instant::now(),
            accumulated_time: Duration::ZERO,
        }
    }

    #[inline]
    fn pass_event(&mut self, event: Event<()>, event_loop: &ActiveEventLoop) -> Result<(), ()> {
        if let Some((app, _)) = self.context.get() {
            handle_error(app.handle(event), event_loop)
        } else {
            Ok(())
        }
    }
}

impl<A: App<D>, D> ApplicationHandler for AppHandler<A, D> {
    #[inline]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if handle_error(self.context.construct_ctx(event_loop), event_loop).is_err() {
            return;
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
        if let Some((app, ctx)) = self.context.get() {
            if ctx.window.id() == window_id {
                ctx.input.process_event(&event);

                if event == WindowEvent::CloseRequested {
                    event_loop.exit();
                }
            }

            let _ = handle_error(app.handle(Event::WindowEvent { window_id, event }), event_loop);
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
        if let Some((app, ctx)) = self.context.get() {
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
