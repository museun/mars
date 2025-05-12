use std::num::NonZeroU16;

use crate::{BlendMode, Color, Rgba};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PixelData {
    Char(char),
    Str(compact_str::CompactString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pixel {
    pub(crate) data: PixelData,
    pub foreground: Color,
    pub background: Color,
    pub attributes: Option<Attributes>,
}

impl Pixel {
    pub const fn empty() -> Self {
        Self {
            data: PixelData::Char(' '),
            foreground: Color::Default,
            background: Color::Default,
            attributes: None,
        }
    }

    pub const fn dirty() -> Self {
        Self {
            data: PixelData::Char(' '),
            foreground: Color::Default,
            background: Color::Rgba(Rgba(0xFF, 0x00, 0xFF, 0xFF)),
            attributes: None,
        }
    }

    pub const fn new(ch: char) -> Self {
        Self {
            data: PixelData::Char(ch),
            foreground: Color::Default,
            background: Color::Default,
            attributes: None,
        }
    }

    pub const fn const_str(str: &'static str) -> Self {
        Self {
            data: PixelData::Str(compact_str::CompactString::const_new(str)),
            foreground: Color::Default,
            background: Color::Default,
            attributes: None,
        }
    }

    #[deprecated(note = "this is just here to figure something out")]
    pub fn data(&self) -> char {
        match self.data {
            PixelData::Char(ch) => ch,
            _ => unreachable!(),
        }
    }

    pub fn new_str(str: &str) -> Self {
        Self {
            data: PixelData::Str(compact_str::CompactString::new(str)),
            foreground: Color::Default,
            background: Color::Default,
            attributes: None,
        }
    }

    pub fn fg(mut self, fg: impl Into<Color>) -> Self {
        self.foreground = fg.into();
        self
    }

    pub fn bg(mut self, bg: impl Into<Color>) -> Self {
        self.background = bg.into();
        self
    }

    pub fn set_fg(&mut self, fg: impl Into<Color>) {
        self.foreground = fg.into();
    }

    pub fn set_bg(&mut self, bg: impl Into<Color>) {
        self.background = bg.into();
    }

    pub fn set_attribute(&mut self, attr: impl Into<Option<Attributes>>) {
        let attr: Option<Attributes> = attr.into();
        match (&mut self.attributes, attr) {
            (Some(old), Some(new)) => *old |= new,
            (old, new) => *old = new,
        }
    }

    pub fn merge_mut(&mut self, other: Self) {
        *self = other;
        // if self.foreground.is_transparent() {
        //     self.foreground = other.foreground;
        //     self.data = other.data;
        // }

        // if self.background.is_transparent() {
        //     self.background = other.background
        // }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.merge_mut(other);
        self
    }

    pub(crate) fn blend_bg(old: Color, other: Color, default: Color, mode: BlendMode) -> Color {
        let BlendMode::Blend = mode else {
            return other.get_or_default(default);
        };

        let (left, right) = match (old, other.get_or_default(default)) {
            (Color::Named(left), Color::Rgba(right)) => (left.to_rgb(), right),
            (Color::Rgba(left), Color::Named(right)) => (left, right.to_rgb()),
            (Color::Rgba(left), Color::Rgba(right)) => (left, right),
            _ => return other,
        };
        let mode = Rgba::pick_blend(left, right);
        Color::Rgba(mode(left, right))
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Attributes(pub NonZeroU16);

impl Default for Attributes {
    fn default() -> Self {
        Self::RESET
    }
}

const fn make_u16(d: u16) -> NonZeroU16 {
    NonZeroU16::new(d).unwrap()
}

impl Attributes {
    pub const RESET: Self = Self(make_u16(u16::MAX));
    pub const BOLD: Self = Self(make_u16(1 << 0));
    pub const FAINT: Self = Self(make_u16(1 << 1));
    pub const ITALIC: Self = Self(make_u16(1 << 2));
    pub const UNDERLINE: Self = Self(make_u16(1 << 3));
    pub const BLINK: Self = Self(make_u16(1 << 4));
    pub const REVERSE: Self = Self(make_u16(1 << 6));
    pub const STRIKEOUT: Self = Self(make_u16(1 << 8));
}

impl Attributes {
    pub const fn is_reset(&self) -> bool {
        self.0.get() == 0
    }

    pub const fn is_bold(&self) -> bool {
        self.0.get() & (1 << 0) != 0
    }

    pub const fn is_faint(&self) -> bool {
        self.0.get() & (1 << 1) != 0
    }

    pub const fn is_italic(&self) -> bool {
        self.0.get() & (1 << 2) != 0
    }

    pub const fn is_underline(&self) -> bool {
        self.0.get() & (1 << 3) != 0
    }

    pub const fn is_blink(&self) -> bool {
        self.0.get() & (1 << 4) != 0
    }

    pub const fn is_reverse(&self) -> bool {
        self.0.get() & (1 << 6) != 0
    }

    pub const fn is_strikeout(&self) -> bool {
        self.0.get() & (1 << 8) != 0
    }
}

impl Attributes {
    pub fn as_indexed_bytes(&self) -> impl Iterator<Item = u8> + use<> {
        let data = self.translate();
        let mut pos = 0;
        std::iter::from_fn(move || {
            loop {
                if pos >= u16::BITS {
                    return None;
                }

                let set = (data & (1 << pos)) != 0;
                pos += 1;
                if set {
                    return Some(pos as _);
                }
            }
        })
    }

    fn translate(&self) -> u16 {
        let d = self.0.get();
        d.checked_sub(u16::MAX).unwrap_or(d)
    }
}

impl std::ops::BitAnd for Attributes {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(make_u16(self.translate() & rhs.translate()))
    }
}

impl std::ops::BitAndAssign for Attributes {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOr for Attributes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(make_u16(self.translate() | rhs.translate()))
    }
}

impl std::ops::BitOrAssign for Attributes {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::Not for Attributes {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(make_u16(!self.translate()))
    }
}
