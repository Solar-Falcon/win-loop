use std::rc::Rc;

use fnv::FnvHashMap;
use winit::{
    event::{ElementState, Event, Modifiers, MouseButton, WindowEvent},
    keyboard::{Key, KeyCode, ModifiersKeyState, NamedKey, PhysicalKey},
    window::Window,
};

/// Keyboard modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct KeyMods {
    /// Left "shift" key.
    pub lshift: bool,
    /// Right "shift" key.
    pub rshift: bool,
    /// Left "alt" key.
    pub lalt: bool,
    /// Right "alt" key.
    pub ralt: bool,
    /// Left "control" key.
    pub lcontrol: bool,
    /// Right "control" key.
    pub rcontrol: bool,
    /// Left "super" key. This is the "windows" key on PC and "command" key on Mac.
    pub lsuper: bool,
    /// Right "super" key. This is the "windows" key on PC and "command" key on Mac.
    pub rsuper: bool,
}

impl KeyMods {
    fn update(&mut self, mods: &Modifiers) {
        self.lshift = mods.lshift_state() == ModifiersKeyState::Pressed;
        self.rshift = mods.rshift_state() == ModifiersKeyState::Pressed;
        self.lalt = mods.lalt_state() == ModifiersKeyState::Pressed;
        self.ralt = mods.ralt_state() == ModifiersKeyState::Pressed;
        self.lcontrol = mods.lcontrol_state() == ModifiersKeyState::Pressed;
        self.rcontrol = mods.rcontrol_state() == ModifiersKeyState::Pressed;
        self.lsuper = mods.lsuper_state() == ModifiersKeyState::Pressed;
        self.rsuper = mods.rsuper_state() == ModifiersKeyState::Pressed;
    }
}

/// Input state of a mouse button/keyboard key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputState {
    /// The button has just been pressed.
    Pressed,
    /// The button is being held down.
    Down,
    /// The button has just been released.
    ///
    /// Note that it means that the key has **just** been released, **not** that it isn't held.
    Released,
}

impl InputState {
    /// The state is [`InputState::Pressed`].
    #[inline]
    pub fn is_pressed(&self) -> bool {
        matches!(self, InputState::Pressed)
    }

    /// The state is [`InputState::Pressed`] or [`InputState::Down`].
    #[inline]
    pub fn is_any_down(&self) -> bool {
        matches!(self, InputState::Pressed | InputState::Down)
    }

    /// The state is [`InputState::Released`].
    #[inline]
    pub fn is_released(&self) -> bool {
        matches!(self, InputState::Released)
    }
}

impl From<ElementState> for InputState {
    #[inline]
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Pressed => InputState::Pressed,
            ElementState::Released => InputState::Released,
        }
    }
}

/// Input handler.
#[derive(Debug)]
pub struct Input {
    pub(crate) window: Rc<Window>,
    mods: KeyMods,
    physical_keys: FnvHashMap<KeyCode, InputState>,
    logical_keys: FnvHashMap<NamedKey, InputState>,
    mouse_buttons: FnvHashMap<MouseButton, InputState>,
    cursor_pos: (f64, f64),
}

impl Input {
    #[inline]
    pub(crate) fn new(window: Rc<Window>) -> Self {
        Self {
            window,
            mods: KeyMods::default(),
            physical_keys: FnvHashMap::default(),
            logical_keys: FnvHashMap::default(),
            mouse_buttons: FnvHashMap::default(),
            cursor_pos: (0., 0.),
        }
    }

    /// Cursor position (from [`WindowEvent::CursorMoved`](https://docs.rs/winit/0.29.5/winit/event/enum.WindowEvent.html#variant.CursorMoved)).
    #[inline]
    pub fn cursor_pos(&self) -> (f64, f64) {
        self.cursor_pos
    }

    /// Get current keyboard modifiers.
    #[inline]
    pub fn key_mods(&self) -> KeyMods {
        self.mods
    }

    /// All pressed physical keys.
    #[inline]
    pub fn physical_keys(&self) -> &FnvHashMap<KeyCode, InputState> {
        &self.physical_keys
    }

    /// Returns `true` if a physical key has just been pressed.
    #[inline]
    pub fn is_physical_key_pressed(&self, scancode: KeyCode) -> bool {
        self.physical_keys
            .get(&scancode)
            .map_or(false, InputState::is_pressed)
    }

    /// Returns `true` if a physical key is down.
    #[inline]
    pub fn is_physical_key_down(&self, scancode: KeyCode) -> bool {
        self.physical_keys
            .get(&scancode)
            .map_or(false, InputState::is_any_down)
    }

    /// Returns `true` if a physical key has just been released.
    #[inline]
    pub fn is_physical_key_released(&self, scancode: KeyCode) -> bool {
        self.physical_keys
            .get(&scancode)
            .map_or(false, InputState::is_released)
    }

    /// All pressed logical keys.
    #[inline]
    pub fn logical_keys(&self) -> &FnvHashMap<NamedKey, InputState> {
        &self.logical_keys
    }

    /// Returns `true` if a logical key has just been pressed.
    #[inline]
    pub fn is_logical_key_pressed(&self, key: NamedKey) -> bool {
        self.logical_keys
            .get(&key)
            .map_or(false, InputState::is_pressed)
    }

    /// Returns `true` if a logical key is down.
    #[inline]
    pub fn is_logical_key_down(&self, key: NamedKey) -> bool {
        self.logical_keys
            .get(&key)
            .map_or(false, InputState::is_any_down)
    }

    /// Returns `true` if a logical key has just been released.
    #[inline]
    pub fn is_logical_key_released(&self, key: NamedKey) -> bool {
        self.logical_keys
            .get(&key)
            .map_or(false, InputState::is_released)
    }

    /// All pressed mouse buttons.
    #[inline]
    pub fn mouse_buttons(&self) -> &FnvHashMap<MouseButton, InputState> {
        &self.mouse_buttons
    }

    /// Returns `true` if a mouse button has just been pressed.
    #[inline]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .map_or(false, InputState::is_pressed)
    }

    /// Returns `true` if a mouse button is down.
    #[inline]
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .map_or(false, InputState::is_any_down)
    }

    /// Returns `true` if a mouse button has just been released.
    #[inline]
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .map_or(false, InputState::is_released)
    }

    pub(crate) fn update_keys(&mut self) {
        self.physical_keys.retain(|_, state| match state {
            InputState::Pressed => {
                *state = InputState::Down;
                true
            }
            InputState::Down => true,
            InputState::Released => false,
        });

        self.logical_keys.retain(|_, state| match state {
            InputState::Pressed => {
                *state = InputState::Down;
                true
            }
            InputState::Down => true,
            InputState::Released => false,
        });

        self.mouse_buttons.retain(|_, state| match state {
            InputState::Pressed => {
                *state = InputState::Down;
                true
            }
            InputState::Down => true,
            InputState::Released => false,
        });
    }

    pub(crate) fn process_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { window_id, event } if *window_id == self.window.id() => {
                match event {
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: false,
                    } if !event.repeat => {
                        if let PhysicalKey::Code(key_code) = event.physical_key {
                            self.physical_keys.insert(key_code, event.state.into());
                        }

                        if let Key::Named(key) = event.logical_key {
                            self.logical_keys.insert(key, event.state.into());
                        }
                    }
                    WindowEvent::ModifiersChanged(mods) => {
                        self.mods.update(mods);
                    }
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        ..
                    } => {
                        self.cursor_pos = (*position).into();
                    }
                    WindowEvent::MouseInput {
                        device_id: _,
                        state,
                        button,
                        ..
                    } => {
                        self.mouse_buttons.insert(*button, (*state).into());
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                device_id: _,
                event: _,
            } => {}
            _ => {}
        }
    }
}
