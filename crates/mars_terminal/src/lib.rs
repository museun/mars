use std::{collections::VecDeque, thread::JoinHandle};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub(crate) hide_cursor: bool,
    pub(crate) mouse_capture: bool,
    pub(crate) ctrl_c_quits: bool,
    pub(crate) use_alt_screen: bool,
    pub(crate) hook_panics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hide_cursor: true,
            mouse_capture: true,
            ctrl_c_quits: true,
            use_alt_screen: true,
            hook_panics: true,
        }
    }

    pub const fn hide_cursor(mut self, hide_cursor: bool) -> Self {
        self.hide_cursor = hide_cursor;
        self
    }

    pub const fn mouse_capture(mut self, mouse_capture: bool) -> Self {
        self.mouse_capture = mouse_capture;
        self
    }

    pub const fn ctrl_c_quits(mut self, ctrl_c_quits: bool) -> Self {
        self.ctrl_c_quits = ctrl_c_quits;
        self
    }

    pub const fn use_alt_screen(mut self, use_alt_screen: bool) -> Self {
        self.use_alt_screen = use_alt_screen;
        self
    }

    pub const fn hook_panics(mut self, hook_panics: bool) -> Self {
        self.hook_panics = hook_panics;
        self
    }
}

pub struct Terminal {
    terminal: termina::PlatformTerminal,
    events: std::sync::mpsc::Receiver<Event>,
    size: Size,
    config: Config,
    _handle: JoinHandle<()>,
}

impl Terminal {
    pub fn create(config: Config) -> std::io::Result<Self> {
        let (tx, events) = std::sync::mpsc::channel();
        let mut terminal = termina::PlatformTerminal::new()?;
        terminal.enter_raw_mode()?;

        let termina::WindowSize { cols, rows } = terminal.get_dimensions()?;
        let size = Size::new(cols as _, rows as _);

        Self::initialize(&mut terminal, config)?;

        let reader = terminal.event_reader();
        let _handle = std::thread::spawn({
            move || {
                const CTRL_C: Keybind = Keybind::char('c').control();

                let mut state = EventState::default();
                'outer: while let Ok(ev) = reader.read(|_| true) {
                    for ev in state.translate(&ev) {
                        let mut was_quit = ev.is_quit();
                        if config.ctrl_c_quits {
                            was_quit ^= ev.is_keybind(&CTRL_C)
                        }

                        if tx.send(ev).is_err() {
                            break 'outer;
                        }

                        if was_quit {
                            break 'outer;
                        }
                    }
                }

                let _ = tx.send(Event::Quit);
            }
        });

        Ok(Self {
            terminal,
            events,
            size,
            config,
            _handle,
        })
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub fn try_read_event(&mut self) -> Option<Event> {
        match self.events.try_recv() {
            Ok(ev) => {
                if let Event::Resize { size } = ev {
                    self.size = size;
                }
                Some(ev)
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Event::Quit),
            _ => None,
        }
    }

    fn initialize(terminal: &mut impl termina::Terminal, config: Config) -> std::io::Result<()> {
        use termina::escape::csi::DecPrivateModeCode as Dec;

        if config.use_alt_screen {
            write!(terminal, "{}", set(Dec::ClearAndEnableAlternateScreen))?;
        }

        if config.hide_cursor {
            write!(terminal, "{}", reset(Dec::ShowCursor))?;
        }

        if config.mouse_capture {
            for mouse in [
                Dec::MouseTracking,
                Dec::ButtonEventMouse,
                Dec::AnyEventMouse,
                Dec::RXVTMouse,
                Dec::SGRMouse,
            ] {
                write!(terminal, "{}", set(mouse))?;
            }
        }

        if config.hook_panics {
            terminal.set_panic_hook(move |out| Self::reset(config, out));
        }

        terminal.flush()?;

        Ok(())
    }

    fn reset(config: Config, terminal: &mut dyn std::io::Write) {
        use termina::escape::csi::DecPrivateModeCode as Dec;

        if config.mouse_capture {
            for mouse in [
                Dec::MouseTracking,
                Dec::ButtonEventMouse,
                Dec::AnyEventMouse,
                Dec::RXVTMouse,
                Dec::SGRMouse,
            ] {
                _ = write!(terminal, "{}", reset(mouse));
                _ = terminal.flush();
            }
        }

        if config.use_alt_screen {
            _ = write!(terminal, "{}", reset(Dec::ClearAndEnableAlternateScreen));
            _ = terminal.flush();
        }

        if config.hide_cursor {
            _ = write!(terminal, "{}", set(Dec::ShowCursor));
            _ = terminal.flush();
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

impl Drop for Terminal {
    fn drop(&mut self) {
        Self::reset(self.config, self);
        _ = self.terminal.enter_cooked_mode();
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
        pos: Position,
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

    pub fn is_keybind(&self, keybind: &Keybind) -> bool {
        let &Self::KeyPress { key, modifiers } = self else {
            return false;
        };
        Keybind { key, modifiers } == *keybind
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
enum DragState {
    Active {
        origin: Position,
        previous: Position,
        button: MouseButton,
    },
    Maybe {
        origin: Position,
    },
    #[default]
    None,
}

#[derive(Debug, Default)]
struct EventState {
    pos: Position,
    drag_state: DragState,
    queue: VecDeque<Event>,
}

impl EventState {
    fn translate(&mut self, event: &termina::event::Event) -> impl IntoIterator<Item = Event> {
        self.process(event);
        self.queue.drain(..)
    }

    fn process(&mut self, event: &termina::event::Event) {
        match event {
            &termina::Event::Key(ke) => self.translate_key(ke),
            &termina::Event::Mouse(me) => self.translate_mouse(me),
            &termina::Event::WindowResized(termina::WindowSize { rows, cols }) => {
                let size = Size::new(cols as u32, rows as u32);
                self.queue.push_back(Event::Resize { size });
            }
            termina::Event::FocusIn | termina::Event::FocusOut | termina::Event::Paste(_) => {}
            _ => {}
        }
    }

    fn translate_key(&mut self, ke: termina::event::KeyEvent) {
        if !matches!(ke.kind, termina::event::KeyEventKind::Press) {
            return;
        }

        let Some(key) = Key::from_termina(ke.code) else {
            return;
        };
        let mut modifiers = KeyModifiers::from_termina(ke.modifiers);
        if let Key::Char(ch) = key {
            if ch.is_uppercase() || ascii_is_uppercase_symbols(ch) {
                modifiers |= KeyModifiers::SHIFT
            }
        }
        self.queue.push_back(Event::KeyPress { key, modifiers });
    }

    fn translate_mouse(&mut self, me: termina::event::MouseEvent) {
        use termina::event::MouseEventKind as T;
        let modifiers = KeyModifiers::from_termina(me.modifiers);
        let pos = Position::new(me.column as _, me.row as _);
        self.pos = pos;

        let ev = match me.kind {
            T::Down(button) => {
                if let state @ DragState::None = &mut self.drag_state {
                    *state = DragState::Maybe { origin: pos };
                };

                Event::MousePress {
                    button: MouseButton::from_termina(button),
                    modifiers,
                    pos,
                    down: true,
                }
            }

            T::Up(button) => {
                let button = MouseButton::from_termina(button);
                if let DragState::Active {
                    origin,
                    button: old,
                    ..
                } = self.drag_state
                {
                    if old == button {
                        // soft-reset the state so can we can remove it in the phantom mouse move
                        let _ = std::mem::replace(
                            &mut self.drag_state,
                            DragState::Maybe { origin: pos },
                        );

                        self.queue.push_back(Event::MouseDragRelease {
                            button,
                            modifiers,
                            origin,
                            pos,
                        });
                    }
                }

                Event::MousePress {
                    button,
                    modifiers,
                    pos,
                    down: false,
                }
            }

            T::Drag(button) => {
                let button = MouseButton::from_termina(button);
                match self.drag_state {
                    DragState::Active {
                        origin,
                        ref mut previous,
                        button,
                    } => {
                        let previous = std::mem::replace(previous, pos);
                        if previous == pos {
                            return;
                        }
                        Event::MouseDragHeld {
                            button,
                            modifiers,
                            origin,
                            pos,
                            delta: previous.delta(pos),
                        }
                    }

                    DragState::Maybe { origin } => {
                        self.drag_state = DragState::Active {
                            origin,
                            previous: pos,
                            button,
                        };
                        Event::MouseDragHeld {
                            button,
                            modifiers,
                            origin,
                            pos,
                            delta: origin.delta(pos),
                        }
                    }

                    DragState::None => Event::MouseDragHeld {
                        button,
                        modifiers,
                        origin: pos,
                        pos,
                        delta: <Delta<i32>>::ZERO,
                    },
                }
            }

            T::Moved => {
                if let DragState::Maybe { origin } = std::mem::take(&mut self.drag_state) {
                    if origin == pos {
                        return;
                    }
                }
                Event::MouseMove { pos, modifiers }
            }

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

        self.queue.push_back(ev);
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

// TODO make a note that auto-casefolding is only done for 7-bit ASCII
const fn ascii_is_uppercase_symbols(ch: char) -> bool {
    matches!(
        ch,
        '~' | '!'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '('
            | ')'
            | '_'
            | '+'
            | '{'
            | '}'
            | '|'
            | '"'
            | '<'
            | '>'
            | '?'
    )
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub key: Key,
    pub modifiers: KeyModifiers,
}

impl Keybind {
    pub const fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub const fn char(char: char) -> Self {
        let mut modifiers = KeyModifiers::NONE;
        if char.is_uppercase() || ascii_is_uppercase_symbols(char) {
            modifiers.0 |= KeyModifiers::SHIFT.0;
        }
        Self {
            key: Key::Char(char),
            modifiers,
        }
    }

    pub const fn shift(mut self) -> Self {
        self.modifiers.0 |= KeyModifiers::SHIFT.0;
        self
    }

    pub const fn alt(mut self) -> Self {
        self.modifiers.0 |= KeyModifiers::ALT.0;
        self
    }

    pub const fn control(mut self) -> Self {
        self.modifiers.0 |= KeyModifiers::CONTROL.0;
        self
    }

    pub const fn super_key(mut self) -> Self {
        self.modifiers.0 |= KeyModifiers::SUPER.0;
        self
    }

    pub const fn hyper(mut self) -> Self {
        self.modifiers.0 |= KeyModifiers::HYPER.0;
        self
    }

    pub const fn meta(mut self) -> Self {
        self.modifiers.0 |= KeyModifiers::META.0;
        self
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
