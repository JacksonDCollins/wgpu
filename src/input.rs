use gilrs::{EventType, Gilrs, GilrsBuilder};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use strum::EnumIter;
use winit::event::{DeviceEvent, Event, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::lock;

#[derive(Debug)]
pub enum InputEnum {
    GamepadAxis(GamepadAxis),
    GamepadButton(GamepadButton),
    KeyboardButton(KeyboardButton),
    MouseButton(MouseButton),
    MouseAxis(MouseAxis),
}

pub trait Listener: std::fmt::Debug {
    fn hear(&self, input: InputEnum, event: InputState);
}
#[derive(Debug)]
pub struct Input {
    inner: Arc<Inner>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                guard: Mutex::new(InnerMut {
                    inputs: HashMap::new(),
                    listeners: HashMap::new(),
                    gilrs: GilrsBuilder::new().set_update_state(false).build().unwrap(), // PANIC
                }),
                updated: AtomicBool::new(false),
            }),
        }
    }

    pub fn register_listener<A: 'static>(&self, listener: Arc<dyn Listener>) {
        lock!(self.inner.guard)
            .listeners
            .entry(TypeId::of::<A>())
            .or_insert_with(|| Vec::new())
            .push(listener);
    }

    pub fn get_f32<I: InputMarker>(&self, input: I) -> InputState {
        Self::get_input(&lock!(self.inner.guard), input)
    }

    pub fn get_bool<I: InputMarker>(&self, input: I) -> bool {
        Self::get_input(&lock!(self.inner.guard), input) != f32::default()
    }

    fn get_input<I: InputMarker>(lock: &MutexGuard<InnerMut>, input: I) -> InputState {
        lock.inputs
            .get(&TypeId::of::<I>())
            .and_then(|i| i.downcast_ref::<HashMap<I, InputState>>())
            .and_then(|i| i.get(&input))
            .map(|i| *i)
            .unwrap_or_default()
    }

    fn set_input<I: InputMarker>(
        lock: &mut MutexGuard<InnerMut>,
        input: I,
        state: InputState,
    ) -> () {
        lock.inputs
            .entry(TypeId::of::<I>())
            .or_insert_with(|| Box::new(HashMap::<I, InputState>::new()))
            .downcast_mut::<HashMap<I, InputState>>()
            .unwrap() // PANIC should never happen
            .insert(input, state);

        Self::yell_listeners(lock, input, state);
    }

    fn set_inputs<I: InputMarker, It: IntoIterator<Item = (I, InputState)>>(
        lock: &mut MutexGuard<InnerMut>,
        iter: It,
    ) -> () {
        lock.inputs
            .entry(TypeId::of::<I>())
            .or_insert_with(|| Box::new(HashMap::<I, InputState>::new()))
            .downcast_mut::<HashMap<I, InputState>>()
            .unwrap() // PANIC should never happen
            .extend(iter);
    }

    pub fn event(&mut self, event: &Event<()>) -> () {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        Self::set_input(
                            &mut lock!(self.inner.guard),
                            KeyboardButton::from_winit(key),
                            ((event.state == winit::event::ElementState::Pressed) as i32) as f32,
                        );
                    }
                }
                &WindowEvent::MouseInput { state, button, .. } => {
                    Self::set_input(
                        &mut lock!(self.inner.guard),
                        MouseButton::from_winit(button),
                        ((state == winit::event::ElementState::Pressed) as i32) as f32,
                    );
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let (x, y) = match delta {
                        &MouseScrollDelta::LineDelta(x, y) => (x, y),
                        MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                    };
                    Self::set_inputs(
                        &mut lock!(self.inner.guard),
                        [(MouseAxis::WheelDeltaX, x), (MouseAxis::WheelDeltaY, y)],
                    );
                }
                WindowEvent::CursorMoved { position, .. } => {
                    Self::set_inputs(
                        &mut lock!(self.inner.guard),
                        [
                            (MouseAxis::PositionX, position.x as f32),
                            (MouseAxis::PositionY, position.y as f32),
                        ],
                    );
                }
                _ => (),
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                Self::set_inputs(
                    &mut lock!(self.inner.guard),
                    [
                        (MouseAxis::PositionDeltaX, delta.0 as f32),
                        (MouseAxis::PositionDeltaY, delta.1 as f32),
                    ],
                );
            }
            _ => (),
        }
        if self.inner.updated.fetch_or(false, Ordering::Relaxed) {
            let mut lock = lock!(self.inner.guard);
            while let Some(event) = lock.gilrs.next_event() {
                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        Self::set_input(
                            &mut lock,
                            GamepadButton::from_gilrs(button),
                            (true as i32) as f32,
                        );
                    }
                    EventType::ButtonReleased(button, _) => {
                        Self::set_input(
                            &mut lock,
                            GamepadButton::from_gilrs(button),
                            (false as i32) as f32,
                        );
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        Self::set_input(&mut lock, GamepadAxis::from_gilrs(axis), value);
                    }
                    _ => (),
                }
            }
            self.inner.updated.store(true, Ordering::Relaxed);
        }
    }

    fn yell_listeners<I: InputMarker>(
        lock: &mut MutexGuard<InnerMut>,
        input: I,
        state: InputState,
    ) {
        lock.listeners
            .get(&TypeId::of::<I>())
            .unwrap_or(&Vec::new())
            .iter()
            .for_each(|listener| listener.hear(input.into(), state));
    }

    pub fn reset(&mut self) -> () {
        self.inner.updated.store(false, Ordering::Relaxed);
        let mut lock = lock!(self.inner.guard);
        Self::set_inputs(
            &mut lock,
            [
                (MouseAxis::PositionDeltaX, 0.0),
                (MouseAxis::PositionDeltaY, 0.0),
                (MouseAxis::WheelDeltaX, 0.0),
                (MouseAxis::WheelDeltaY, 0.0),
            ],
        );
    }
}

impl Clone for Input {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

//
// Inner
//
#[derive(Debug)]
struct Inner {
    guard: Mutex<InnerMut>,
    updated: AtomicBool,
}

//
// InnerMut
//
#[derive(Debug)]
struct InnerMut {
    /// `Any = HashMap<I: InputMarker, InputState>`
    inputs: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
    listeners: HashMap<TypeId, Vec<Arc<dyn Listener>>>,
    gilrs: Gilrs,
}

//
// InputMarker
//

pub trait InputMarker: Clone + Copy + PartialEq + Eq + Hash + Send + Sync + 'static {
    fn into(self) -> InputEnum;
}

//
// InputState
//

pub type InputState = f32;

//
// MouseAxis
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAxis {
    PositionX,
    PositionY,
    PositionDeltaX,
    PositionDeltaY,
    // MAYBE WheelX,
    // MAYBE WheelY,
    WheelDeltaX,
    WheelDeltaY,
}

impl InputMarker for MouseAxis {
    fn into(self) -> InputEnum {
        InputEnum::MouseAxis(self)
    }
}

//
// GamepadAxis
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ,
    DPadX,
    DPadY,
    Other,
}

impl GamepadAxis {
    fn from_gilrs(gilrs: gilrs::Axis) -> Self {
        match gilrs {
            gilrs::Axis::LeftStickX => Self::LeftStickX,
            gilrs::Axis::LeftStickY => Self::LeftStickY,
            gilrs::Axis::LeftZ => Self::LeftZ,
            gilrs::Axis::RightStickX => Self::RightStickX,
            gilrs::Axis::RightStickY => Self::RightStickY,
            gilrs::Axis::RightZ => Self::RightZ,
            gilrs::Axis::DPadX => Self::DPadX,
            gilrs::Axis::DPadY => Self::DPadY,
            gilrs::Axis::Unknown => Self::Other,
        }
    }
}

impl InputMarker for GamepadAxis {
    fn into(self) -> InputEnum {
        InputEnum::GamepadAxis(self)
    }
}

//
// MouseButton
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl MouseButton {
    fn from_winit(winit: winit::event::MouseButton) -> Self {
        match winit {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Back => Self::Back,
            winit::event::MouseButton::Forward => Self::Forward,
            winit::event::MouseButton::Other(btn) => Self::Other(btn),
        }
    }
}

impl InputMarker for MouseButton {
    fn into(self) -> InputEnum {
        InputEnum::MouseButton(self)
    }
}

//
// GamepadButton
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    C,
    Z,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Other,
}

impl GamepadButton {
    fn from_gilrs(gilrs: gilrs::Button) -> Self {
        match gilrs {
            gilrs::Button::South => Self::South,
            gilrs::Button::East => Self::East,
            gilrs::Button::North => Self::North,
            gilrs::Button::West => Self::West,
            gilrs::Button::C => Self::C,
            gilrs::Button::Z => Self::Z,
            gilrs::Button::LeftTrigger => Self::LeftTrigger,
            gilrs::Button::LeftTrigger2 => Self::LeftTrigger2,
            gilrs::Button::RightTrigger => Self::RightTrigger,
            gilrs::Button::RightTrigger2 => Self::RightTrigger2,
            gilrs::Button::Select => Self::Select,
            gilrs::Button::Start => Self::Start,
            gilrs::Button::Mode => Self::Mode,
            gilrs::Button::LeftThumb => Self::LeftThumb,
            gilrs::Button::RightThumb => Self::RightThumb,
            gilrs::Button::DPadUp => Self::DPadUp,
            gilrs::Button::DPadDown => Self::DPadDown,
            gilrs::Button::DPadLeft => Self::DPadLeft,
            gilrs::Button::DPadRight => Self::DPadRight,
            gilrs::Button::Unknown => Self::Other,
        }
    }
}

impl InputMarker for GamepadButton {
    fn into(self) -> InputEnum {
        InputEnum::GamepadButton(self)
    }
}

//
// KeyboardButton
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
pub enum KeyboardButton {
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    SuperLeft,
    SuperRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Meta,
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}

impl KeyboardButton {
    fn from_winit(winit: winit::keyboard::KeyCode) -> Self {
        match winit {
            KeyCode::Backquote => Self::Backquote,
            KeyCode::Backslash => Self::Backslash,
            KeyCode::BracketLeft => Self::BracketLeft,
            KeyCode::BracketRight => Self::BracketRight,
            KeyCode::Comma => Self::Comma,
            KeyCode::Digit0 => Self::Digit0,
            KeyCode::Digit1 => Self::Digit1,
            KeyCode::Digit2 => Self::Digit2,
            KeyCode::Digit3 => Self::Digit3,
            KeyCode::Digit4 => Self::Digit4,
            KeyCode::Digit5 => Self::Digit5,
            KeyCode::Digit6 => Self::Digit6,
            KeyCode::Digit7 => Self::Digit7,
            KeyCode::Digit8 => Self::Digit8,
            KeyCode::Digit9 => Self::Digit9,
            KeyCode::Equal => Self::Equal,
            KeyCode::IntlBackslash => Self::IntlBackslash,
            KeyCode::IntlRo => Self::IntlRo,
            KeyCode::IntlYen => Self::IntlYen,
            KeyCode::KeyA => Self::KeyA,
            KeyCode::KeyB => Self::KeyB,
            KeyCode::KeyC => Self::KeyC,
            KeyCode::KeyD => Self::KeyD,
            KeyCode::KeyE => Self::KeyE,
            KeyCode::KeyF => Self::KeyF,
            KeyCode::KeyG => Self::KeyG,
            KeyCode::KeyH => Self::KeyH,
            KeyCode::KeyI => Self::KeyI,
            KeyCode::KeyJ => Self::KeyJ,
            KeyCode::KeyK => Self::KeyK,
            KeyCode::KeyL => Self::KeyL,
            KeyCode::KeyM => Self::KeyM,
            KeyCode::KeyN => Self::KeyN,
            KeyCode::KeyO => Self::KeyO,
            KeyCode::KeyP => Self::KeyP,
            KeyCode::KeyQ => Self::KeyQ,
            KeyCode::KeyR => Self::KeyR,
            KeyCode::KeyS => Self::KeyS,
            KeyCode::KeyT => Self::KeyT,
            KeyCode::KeyU => Self::KeyU,
            KeyCode::KeyV => Self::KeyV,
            KeyCode::KeyW => Self::KeyW,
            KeyCode::KeyX => Self::KeyX,
            KeyCode::KeyY => Self::KeyY,
            KeyCode::KeyZ => Self::KeyZ,
            KeyCode::Minus => Self::Minus,
            KeyCode::Period => Self::Period,
            KeyCode::Quote => Self::Quote,
            KeyCode::Semicolon => Self::Semicolon,
            KeyCode::Slash => Self::Slash,
            KeyCode::AltLeft => Self::AltLeft,
            KeyCode::AltRight => Self::AltRight,
            KeyCode::Backspace => Self::Backspace,
            KeyCode::CapsLock => Self::CapsLock,
            KeyCode::ContextMenu => Self::ContextMenu,
            KeyCode::ControlLeft => Self::ControlLeft,
            KeyCode::ControlRight => Self::ControlRight,
            KeyCode::Enter => Self::Enter,
            KeyCode::SuperLeft => Self::SuperLeft,
            KeyCode::SuperRight => Self::SuperRight,
            KeyCode::ShiftLeft => Self::ShiftLeft,
            KeyCode::ShiftRight => Self::ShiftRight,
            KeyCode::Space => Self::Space,
            KeyCode::Tab => Self::Tab,
            KeyCode::Convert => Self::Convert,
            KeyCode::KanaMode => Self::KanaMode,
            KeyCode::Lang1 => Self::Lang1,
            KeyCode::Lang2 => Self::Lang2,
            KeyCode::Lang3 => Self::Lang3,
            KeyCode::Lang4 => Self::Lang4,
            KeyCode::Lang5 => Self::Lang5,
            KeyCode::NonConvert => Self::NonConvert,
            KeyCode::Delete => Self::Delete,
            KeyCode::End => Self::End,
            KeyCode::Help => Self::Help,
            KeyCode::Home => Self::Home,
            KeyCode::Insert => Self::Insert,
            KeyCode::PageDown => Self::PageDown,
            KeyCode::PageUp => Self::PageUp,
            KeyCode::ArrowDown => Self::ArrowDown,
            KeyCode::ArrowLeft => Self::ArrowLeft,
            KeyCode::ArrowRight => Self::ArrowRight,
            KeyCode::ArrowUp => Self::ArrowUp,
            KeyCode::NumLock => Self::NumLock,
            KeyCode::Numpad0 => Self::Numpad0,
            KeyCode::Numpad1 => Self::Numpad1,
            KeyCode::Numpad2 => Self::Numpad2,
            KeyCode::Numpad3 => Self::Numpad3,
            KeyCode::Numpad4 => Self::Numpad4,
            KeyCode::Numpad5 => Self::Numpad5,
            KeyCode::Numpad6 => Self::Numpad6,
            KeyCode::Numpad7 => Self::Numpad7,
            KeyCode::Numpad8 => Self::Numpad8,
            KeyCode::Numpad9 => Self::Numpad9,
            KeyCode::NumpadAdd => Self::NumpadAdd,
            KeyCode::NumpadBackspace => Self::NumpadBackspace,
            KeyCode::NumpadClear => Self::NumpadClear,
            KeyCode::NumpadClearEntry => Self::NumpadClearEntry,
            KeyCode::NumpadComma => Self::NumpadComma,
            KeyCode::NumpadDecimal => Self::NumpadDecimal,
            KeyCode::NumpadDivide => Self::NumpadDivide,
            KeyCode::NumpadEnter => Self::NumpadEnter,
            KeyCode::NumpadEqual => Self::NumpadEqual,
            KeyCode::NumpadHash => Self::NumpadHash,
            KeyCode::NumpadMemoryAdd => Self::NumpadMemoryAdd,
            KeyCode::NumpadMemoryClear => Self::NumpadMemoryClear,
            KeyCode::NumpadMemoryRecall => Self::NumpadMemoryRecall,
            KeyCode::NumpadMemoryStore => Self::NumpadMemoryStore,
            KeyCode::NumpadMemorySubtract => Self::NumpadMemorySubtract,
            KeyCode::NumpadMultiply => Self::NumpadMultiply,
            KeyCode::NumpadParenLeft => Self::NumpadParenLeft,
            KeyCode::NumpadParenRight => Self::NumpadParenRight,
            KeyCode::NumpadStar => Self::NumpadStar,
            KeyCode::NumpadSubtract => Self::NumpadSubtract,
            KeyCode::Escape => Self::Escape,
            KeyCode::Fn => Self::Fn,
            KeyCode::FnLock => Self::FnLock,
            KeyCode::PrintScreen => Self::PrintScreen,
            KeyCode::ScrollLock => Self::ScrollLock,
            KeyCode::Pause => Self::Pause,
            KeyCode::BrowserBack => Self::BrowserBack,
            KeyCode::BrowserFavorites => Self::BrowserFavorites,
            KeyCode::BrowserForward => Self::BrowserForward,
            KeyCode::BrowserHome => Self::BrowserHome,
            KeyCode::BrowserRefresh => Self::BrowserRefresh,
            KeyCode::BrowserSearch => Self::BrowserSearch,
            KeyCode::BrowserStop => Self::BrowserStop,
            KeyCode::Eject => Self::Eject,
            KeyCode::LaunchApp1 => Self::LaunchApp1,
            KeyCode::LaunchApp2 => Self::LaunchApp2,
            KeyCode::LaunchMail => Self::LaunchMail,
            KeyCode::MediaPlayPause => Self::MediaPlayPause,
            KeyCode::MediaSelect => Self::MediaSelect,
            KeyCode::MediaStop => Self::MediaStop,
            KeyCode::MediaTrackNext => Self::MediaTrackNext,
            KeyCode::MediaTrackPrevious => Self::MediaTrackPrevious,
            KeyCode::Power => Self::Power,
            KeyCode::Sleep => Self::Sleep,
            KeyCode::AudioVolumeDown => Self::AudioVolumeDown,
            KeyCode::AudioVolumeMute => Self::AudioVolumeMute,
            KeyCode::AudioVolumeUp => Self::AudioVolumeUp,
            KeyCode::WakeUp => Self::WakeUp,
            KeyCode::Meta => Self::Meta,
            KeyCode::Hyper => Self::Hyper,
            KeyCode::Turbo => Self::Turbo,
            KeyCode::Abort => Self::Abort,
            KeyCode::Resume => Self::Resume,
            KeyCode::Suspend => Self::Suspend,
            KeyCode::Again => Self::Again,
            KeyCode::Copy => Self::Copy,
            KeyCode::Cut => Self::Cut,
            KeyCode::Find => Self::Find,
            KeyCode::Open => Self::Open,
            KeyCode::Paste => Self::Paste,
            KeyCode::Props => Self::Props,
            KeyCode::Select => Self::Select,
            KeyCode::Undo => Self::Undo,
            KeyCode::Hiragana => Self::Hiragana,
            KeyCode::Katakana => Self::Katakana,
            KeyCode::F1 => Self::F1,
            KeyCode::F2 => Self::F2,
            KeyCode::F3 => Self::F3,
            KeyCode::F4 => Self::F4,
            KeyCode::F5 => Self::F5,
            KeyCode::F6 => Self::F6,
            KeyCode::F7 => Self::F7,
            KeyCode::F8 => Self::F8,
            KeyCode::F9 => Self::F9,
            KeyCode::F10 => Self::F10,
            KeyCode::F11 => Self::F11,
            KeyCode::F12 => Self::F12,
            KeyCode::F13 => Self::F13,
            KeyCode::F14 => Self::F14,
            KeyCode::F15 => Self::F15,
            KeyCode::F16 => Self::F16,
            KeyCode::F17 => Self::F17,
            KeyCode::F18 => Self::F18,
            KeyCode::F19 => Self::F19,
            KeyCode::F20 => Self::F20,
            KeyCode::F21 => Self::F21,
            KeyCode::F22 => Self::F22,
            KeyCode::F23 => Self::F23,
            KeyCode::F24 => Self::F24,
            KeyCode::F25 => Self::F25,
            KeyCode::F26 => Self::F26,
            KeyCode::F27 => Self::F27,
            KeyCode::F28 => Self::F28,
            KeyCode::F29 => Self::F29,
            KeyCode::F30 => Self::F30,
            KeyCode::F31 => Self::F31,
            KeyCode::F32 => Self::F32,
            KeyCode::F33 => Self::F33,
            KeyCode::F34 => Self::F34,
            KeyCode::F35 => Self::F35,
            key @ _ => panic!("Missing keyboard key '{:?}'", key), // PANIC unsure of which key(s) is(are) missing
        }
    }
}

impl InputMarker for KeyboardButton {
    fn into(self) -> InputEnum {
        InputEnum::KeyboardButton(self)
    }
}

mod macros {
    #[macro_export]
    macro_rules! lock {
        ($guard:expr) => {
            $guard.lock().unwrap()
        };
    }
}
