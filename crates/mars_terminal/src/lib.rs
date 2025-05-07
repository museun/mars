use std::{collections::HashMap, thread::JoinHandle};

use mars_math::{Delta, Position, Size};
use termina::Terminal as _;

const fn set(f: termina::escape::csi::DecPrivateModeCode) -> termina::escape::csi::Csi {
    termina::escape::csi::Csi::Mode(termina::escape::csi::Mode::SetDecPrivateMode(
        termina::escape::csi::DecPrivateMode::Code(f),
    ))
}

const fn reset(f: termina::escape::csi::DecPrivateModeCode) -> termina::escape::csi::Csi {
    termina::escape::csi::Csi::Mode(termina::escape::csi::Mode::ResetDecPrivateMode(
        termina::escape::csi::DecPrivateMode::Code(f),
    ))
}

fn modify(
    d: &mut impl std::io::Write,
    mode: fn(termina::escape::csi::DecPrivateModeCode) -> termina::escape::csi::Csi,
) -> std::io::Result<()> {
    use termina::escape::csi::DecPrivateModeCode as Dec;
    for m in [
        Dec::MouseTracking,
        Dec::ButtonEventMouse,
        Dec::AnyEventMouse,
        Dec::RXVTMouse,
        Dec::SGRMouse,
    ] {
        write!(d, "{}", mode(m))?;
    }
    d.flush()
}

fn kitty<const N: usize>(
    d: &mut impl std::io::Write,
    flags: [termina::escape::csi::Keyboard; N],
) -> std::io::Result<()> {
    for flag in flags {
        write!(d, "{}", termina::escape::csi::Csi::Keyboard(flag))?;
    }
    d.flush()
}

fn build(d: &mut impl std::io::Write) -> std::io::Result<()> {
    modify(d, set)
    // kitty(
    //     d,
    //     [termina::escape::csi::Keyboard::PushFlags(
    //         termina::escape::csi::KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
    //             | termina::escape::csi::KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
    //             | termina::escape::csi::KittyKeyboardFlags::REPORT_EVENT_TYPES,
    //     )],
    // )
}

fn teardown(d: &mut impl std::io::Write) -> std::io::Result<()> {
    modify(d, reset)
    // kitty(d, [termina::escape::csi::Keyboard::PopFlags(1)])
}

pub struct Terminal {
    sender: std::sync::mpsc::Sender<Request>,
    requests: std::sync::mpsc::Receiver<Request>, // how are we gonna handle this?
    terminal: termina::PlatformTerminal,
    events: std::sync::mpsc::Receiver<Event>,
    size: Size,
    _handle: JoinHandle<()>,
}

impl Terminal {
    pub fn create() -> std::io::Result<Self> {
        let (sender, requests) = std::sync::mpsc::channel();
        let (tx, events) = std::sync::mpsc::channel();
        let terminal = termina::PlatformTerminal::new()?;

        let (h, w) = terminal.get_dimensions()?;
        let size = Size::new(w as _, h as _);

        let reader = terminal.event_reader();
        let _handle = std::thread::spawn(move || {
            let mut state = EventState::default();
            while let Ok(ev) = reader.read(|_| true) {
                let Some(ev) = Event::translate(ev, &mut state) else {
                    continue;
                };
                let was_quit = ev.is_quit();
                if tx.send(ev).is_err() {
                    break;
                }
                if was_quit {
                    break;
                }
            }
            let _ = tx.send(Event::Quit);
        });

        Ok(Self {
            sender,
            requests,
            terminal,
            events,
            size,
            _handle,
        })
    }

    pub fn size(&self) -> Size {
        todo!();
    }

    pub fn try_read_event(&mut self) -> Option<Event> {
        match self.events.try_recv() {
            Ok(ev) => {
                if let Event::Resize { size } = ev {
                    self.size = size;
                }
                // if is ctrl_c keybind and configured, send quit as well
                Some(ev)
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Event::Quit),
            _ => None,
        }
    }
}

impl std::io::Write for Terminal {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.terminal.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.terminal.flush()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    KeyPress {
        key: Key,
        modifiers: KeyModifiers,
    },
    MouseMove {
        pos: Position,
        modifiers: KeyModifiers,
    },
    MouseScroll {
        delta: Delta<i32>,
    },
    MousePress {
        button: MouseButton,
        modifiers: KeyModifiers,
        pos: Position,
        down: bool,
    },
    MouseDragStart {
        button: MouseButton,
        modifiers: KeyModifiers,
        origin: Position,
    },
    MouseDragHeld {
        button: MouseButton,
        modifiers: KeyModifiers,
        origin: Position,
        pos: Position,
        delta: Delta<i32>,
    },
    MouseDragRelease {
        button: MouseButton,
        modifiers: KeyModifiers,
        origin: Position,
    },
    Resize {
        size: Size,
    },
    Quit,
}

impl Event {
    pub const fn is_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }

    fn translate(ev: termina::Event, state: &mut EventState) -> Option<Self> {
        match ev {
            termina::Event::Key(ke) => translate_key(ke, state),
            termina::Event::Mouse(me) => translate_mouse(me, state),
            termina::Event::WindowResized { rows, cols } => {
                let size = Size::new(cols as _, rows as _);
                Some(Event::Resize { size })
            }
            termina::Event::FocusIn | termina::Event::FocusOut | termina::Event::Paste(_) => None,
            _ => None,
        }
    }
}

fn translate_mouse(me: termina::event::MouseEvent, state: &mut EventState) -> Option<Event> {
    use termina::event::MouseEventKind as T;
    let modifiers = KeyModifiers::from_termina(me.modifiers);
    let pos = Position::new(me.column as _, me.row as _);
    state.pos = pos;

    let ev = match me.kind {
        T::Down(..) | T::Up(..) | T::Drag(..) => state.update(me),
        T::Moved => Event::MouseMove { pos, modifiers },
        T::ScrollDown => Event::MouseScroll {
            delta: Delta::new(0, -1),
        },
        T::ScrollUp => Event::MouseScroll {
            delta: Delta::new(0, 1),
        },
        T::ScrollLeft => Event::MouseScroll {
            delta: Delta::new(-1, 0),
        },
        T::ScrollRight => Event::MouseScroll {
            delta: Delta::new(1, 0),
        },
    };

    Some(ev)
}

fn translate_key(ke: termina::event::KeyEvent, _state: &mut EventState) -> Option<Event> {
    // TODO progressively support kitty so we can get Release events as well
    if !matches!(ke.kind, termina::event::KeyEventKind::Press) {
        return None;
    }

    let key = Key::from_termina(ke.code)?;
    let modifiers = KeyModifiers::from_termina(ke.modifiers);
    Some(Event::KeyPress { key, modifiers })
}

#[derive(Default)]
struct EventState {
    pos: Position,
    drag_start: Option<Position>,
    active: Option<MouseButton>,
}

impl EventState {
    fn update(&mut self, me: termina::event::MouseEvent) -> Event {
        use termina::event::MouseEventKind as T;
        let pos = Position::new(me.column as _, me.row as _);
        let modifiers = KeyModifiers::from_termina(me.modifiers);

        match me.kind {
            T::Down(button) => {
                let button = MouseButton::from_termina(button);
                self.active = Some(button);
                self.drag_start = Some(pos);
                Event::MousePress {
                    button,
                    modifiers,
                    pos,
                    down: true,
                }
            }

            T::Up(button) => {
                let button = MouseButton::from_termina(button);
                if let Some(button) = self.active {
                    let origin = self.drag_start.unwrap_or(pos);
                    self.active = None;
                    self.drag_start = None;
                    Event::MouseDragRelease {
                        button,
                        modifiers,
                        origin,
                    }
                } else {
                    Event::MousePress {
                        button,
                        modifiers,
                        pos,
                        down: false,
                    }
                }
            }

            T::Drag(button) => {
                let button = MouseButton::from_termina(button);
                let origin = self.drag_start.unwrap_or(pos);
                if self.active.is_none() {
                    self.active = Some(button);
                    self.drag_start = Some(pos);
                    return Event::MousePress {
                        button,
                        modifiers,
                        pos,
                        down: true,
                    };
                }

                if pos == origin {
                    Event::MouseDragStart {
                        button,
                        modifiers,
                        origin,
                    }
                } else {
                    Event::MouseDragHeld {
                        button,
                        modifiers,
                        origin,
                        pos,
                        delta: pos.delta(origin),
                    }
                }
            }

            T::Moved => {
                self.active.take();
                self.drag_start.take();
                Event::MouseMove { pos, modifiers }
            }

            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ButtonState {
    JustDown,
    Down,
    JustUp,
    Up,
}

impl ButtonState {
    fn settle(&mut self) {
        *self = match self {
            Self::JustDown => Self::Down,
            Self::JustUp => Self::Up,
            _ => return,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MouseButton {
    Primary,
    Secondary,
    Middle,
}

impl MouseButton {
    fn from_termina(button: termina::event::MouseButton) -> Self {
        match button {
            termina::event::MouseButton::Left => Self::Primary,
            termina::event::MouseButton::Right => Self::Secondary,
            termina::event::MouseButton::Middle => Self::Middle,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyModifiers(u8);
impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const CONTROL: Self = Self(1 << 3);
    pub const SUPER: Self = Self(1 << 4);
    pub const HYPER: Self = Self(1 << 5);
    pub const META: Self = Self(1 << 5);
}

impl std::ops::BitAnd for KeyModifiers {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for KeyModifiers {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::Not for KeyModifiers {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl KeyModifiers {
    fn from_termina(modifiers: termina::event::Modifiers) -> Self {
        const THEIRS: [termina::event::Modifiers; 6] = [
            termina::event::Modifiers::SHIFT,
            termina::event::Modifiers::ALT,
            termina::event::Modifiers::CONTROL,
            termina::event::Modifiers::SUPER,
            termina::event::Modifiers::HYPER,
            termina::event::Modifiers::META,
        ];

        THEIRS
            .into_iter()
            .fold(Self::NONE, |this, m| this | Self((modifiers & m).bits()))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    BackTab,
    PageUp,
    PageDown,
    Insert,
    Delete,
    KeypadBegin,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    Function(u8),
    Null,
}

impl Key {
    fn from_termina(key: termina::event::KeyCode) -> Option<Self> {
        use termina::event::KeyCode as T;
        let this = match key {
            T::Char(ch) => Self::Char(ch),
            T::Enter => Self::Enter,
            T::Backspace => Self::Backspace,
            T::Tab => Self::Tab,
            T::Escape => Self::Escape,
            T::Left => Self::Left,
            T::Right => Self::Right,
            T::Up => Self::Up,
            T::Down => Self::Down,
            T::Home => Self::Home,
            T::End => Self::End,
            T::BackTab => Self::BackTab,
            T::PageUp => Self::PageUp,
            T::PageDown => Self::PageDown,
            T::Insert => Self::Insert,
            T::Delete => Self::Delete,
            T::KeypadBegin => Self::KeypadBegin,
            T::CapsLock => Self::CapsLock,
            T::ScrollLock => Self::ScrollLock,
            T::NumLock => Self::NumLock,
            T::PrintScreen => Self::PrintScreen,
            T::Pause => Self::Pause,
            T::Menu => Self::Menu,
            T::Null => Self::Null,
            T::Function(f) => Self::Function(f),
            _ => return None,
        };
        Some(this)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum Action {
    #[default]
    Continue,
    Quit,
}

enum Request {
    SetTitle(String),
}

#[derive(Clone)]
pub struct Context {
    tx: std::sync::mpsc::Sender<Request>,
}

impl Context {
    pub fn new(term: &Terminal) -> Self {
        Self {
            tx: term.sender.clone(),
        }
    }

    pub fn set_title(&self, title: impl ToString) -> bool {
        self.tx.send(Request::SetTitle(title.to_string())).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use termina::{
        Event, PlatformTerminal,
        event::{KeyCode, KeyEvent},
    };

    use super::*;
    #[test]
    fn asdf() {
        fn defer(f: impl FnOnce()) -> impl Drop {
            struct Defer<F: FnOnce()>(Option<F>);
            impl<F: FnOnce()> Drop for Defer<F> {
                fn drop(&mut self) {
                    let Some(f) = self.0.take() else { return };
                    (f)()
                }
            }
            Defer(Some(f))
        }

        let mut t = PlatformTerminal::new().unwrap();
        build(&mut t).unwrap();
        let e = t.event_reader();

        let _d = defer(|| _ = teardown(&mut std::io::stdout()));

        let mut state = EventState::default();

        while let Ok(ev) = e.read(|_| true) {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = ev
            {
                break;
            };

            if let Event::Key(ke) = ev {
                println!("\r{ke:?}")
            }

            // totally forgot what I was doing getting this set up..
            if let Event::Mouse(me) = ev {
                let e = translate_mouse(me, &mut state);
                println!("\r{e:?}");
            }
        }
    }
}
