#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(missing_docs)]

use cfg_if::cfg_if;
use handler::AppHandler;
use web_time::Duration;
use winit::{
    event::Event,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

mod handler;
mod input;
pub use input::{Input, InputState};

pub use anyhow;
pub use winit;

/// Update context.
#[derive(Debug)]
pub struct Context {
    target_frame_time: Duration,
    max_frame_time: Duration,
    exit: bool,
    delta_time: Duration,
    /// Input handler.
    pub input: Input,
}

impl Context {
    /// Create a new context.
    #[inline]
    pub(crate) fn new(fps: u32, max_frame_time: Duration) -> Self {
        Self {
            target_frame_time: Duration::from_secs_f64(1. / fps as f64),
            max_frame_time,
            delta_time: Duration::ZERO,
            exit: false,
            input: Input::new(),
        }
    }

    /// Time between previous and current update.
    #[inline]
    pub fn frame_time(&self) -> Duration {
        self.delta_time
    }

    /// Set the desired (minimum) time between application updates.
    /// Implemented based on <https://gafferongames.com/post/fix_your_timestep>.
    #[inline]
    pub fn set_target_frame_time(&mut self, time: Duration) {
        self.target_frame_time = time;
    }

    /// Set the desired FPS. Overrides `target_frame_time` since they are inversions of each other.
    #[inline]
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_frame_time = Duration::from_secs_f64(1. / fps as f64);
    }

    /// Set the maximum time between application updates.
    /// The real frame time can be longer, but [`Context::frame_time()`] will not exceed this value.
    /// Implemented based on <https://gafferongames.com/post/fix_your_timestep>.
    #[inline]
    pub fn set_max_frame_time(&mut self, time: Duration) {
        self.max_frame_time = time;
    }

    /// Exit the application.
    #[inline]
    pub fn exit(&mut self) {
        self.exit = true;
    }
}

/// Application trait.
pub trait App {
    /// Application update.
    /// Rate of updates can be set using [`Context`].
    fn update(&mut self, ctx: &mut Context) -> anyhow::Result<()>;

    /// Application render.
    /// Will be called once every frame.
    fn render(&mut self, blending_factor: f64) -> anyhow::Result<()>;

    /// Custom event handler if needed.
    #[inline]
    fn handle(&mut self, _event: Event<()>) -> anyhow::Result<()> {
        Ok(())
    }
}

/// App initialization function.
pub type AppInitFunc<A> = dyn FnOnce(&ActiveEventLoop) -> anyhow::Result<A>;

/// Start the application.
/// `app_init` will be called once during the next [`Event::Resumed`](https://docs.rs/winit/latest/winit/event/enum.Event.html#variant.Resumed).
///
/// Depending on the platform, this function may not return (see <https://docs.rs/winit/latest/winit/event_loop/struct.EventLoop.html#method.run_app>).
/// On web uses <https://docs.rs/winit/latest/wasm32-unknown-unknown/winit/platform/web/trait.EventLoopExtWebSys.html#tymethod.spawn_app> instead of `run_app()`.
pub fn start<A>(
    fps: u32,
    max_frame_time: Duration,
    app_init: Box<AppInitFunc<A>>,
) -> anyhow::Result<()>
where
    A: App,
{
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut handler = AppHandler::new(app_init, fps, max_frame_time);

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;

            event_loop.spawn_app(handler);
        } else {
            event_loop.run_app(&mut handler)?;
        }
    }

    Ok(())
}
