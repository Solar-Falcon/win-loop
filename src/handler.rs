use crate::{App, Context};
use web_time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{Event, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

#[derive(Debug)]
pub struct AppHandler<A: App> {
    context: Context,
    app: A,
    instant: Instant,
    accumulated_time: Duration,
}

impl<A: App> AppHandler<A> {
    #[inline]
    pub fn new(context: Context, app: A) -> Self {
        Self {
            context,
            app,
            instant: Instant::now(),
            accumulated_time: Duration::ZERO,
        }
    }
}

impl<A: App> ApplicationHandler for AppHandler<A> {
    #[inline]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
        if self.context.window().id() == window_id {
            self.context.input.process_event(&event);

            if event == WindowEvent::CloseRequested {
                event_loop.exit();
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

        let mut elapsed = self.instant.elapsed();
        self.instant = Instant::now();

        if elapsed > self.context.max_frame_time {
            elapsed = self.context.max_frame_time;
        }

        self.accumulated_time += elapsed;

        let mut keys_updated = false;

        while self.accumulated_time > self.context.target_frame_time {
            self.context.delta_time = self.context.target_frame_time;

            if handle_error(self.app.update(&mut self.context), event_loop).is_err() {
                return;
            }

            if self.context.exit {
                event_loop.exit();
                return;
            }

            if !keys_updated {
                self.context.input.update_keys();
                keys_updated = true;
            }

            self.accumulated_time = self
                .accumulated_time
                .saturating_sub(self.context.target_frame_time);

            let blending_factor =
                self.accumulated_time.as_secs_f64() / self.context.target_frame_time.as_secs_f64();

            let _ = handle_error(self.app.render(blending_factor), event_loop);
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
