use crate::{Attributes, BlendMode, Color};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub attributes: Option<Attributes>,
    pub blend: BlendMode,
}

impl Default for Style {
    fn default() -> Self {
        Self::empty()
    }
}

impl Style {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            foreground: Color::Default, // is this ideal for things like text?
            background: Color::Default, // is this ideal for things like text?
            attributes: None,
            blend: BlendMode::Replace,
        }
    }

    #[must_use]
    pub fn foreground(mut self, foreground: impl Into<Color>) -> Self {
        self.foreground = foreground.into();
        self
    }

    #[must_use]
    pub fn background(mut self, background: impl Into<Color>) -> Self {
        self.background = background.into();
        self
    }

    #[must_use]
    pub fn attributes(mut self, attributes: impl Into<Option<Attributes>>) -> Self {
        self.attributes = attributes.into();
        self
    }

    #[must_use]
    pub fn blend(mut self, blend: BlendMode) -> Self {
        self.blend = blend;
        self
    }
}
