use std::{
    convert::Infallible,
    io::{Cursor, Write},
    num::NonZeroU16,
    u16,
};

use mars_math::{Position, Size, pos};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PixelData {
    Char(char),
    Str(compact_str::CompactString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pixel {
    data: PixelData,
    foreground: Color,
    background: Color,
    attributes: Option<Attributes>,
}

impl Pixel {
    pub const DEFAULT: Self = Self {
        data: PixelData::Char(' '),
        foreground: Color::Default,
        background: Color::Default,
        attributes: None,
    };

    pub const fn empty() -> Self {
        Self {
            data: PixelData::Char(' '),
            foreground: Color::Transparent,
            background: Color::Transparent,
            attributes: None,
        }
    }

    pub const fn dirty() -> Self {
        Self {
            data: PixelData::Char(' '),
            foreground: Color::Rgba(Rgba(0xFF, 0x00, 0xFF, 0xFF)),
            background: Color::Transparent,
            attributes: None,
        }
    }

    pub const fn new(ch: char) -> Self {
        Self {
            data: PixelData::Char(ch),
            ..Self::DEFAULT
        }
    }

    pub const fn const_str(str: &'static str) -> Self {
        Self {
            data: PixelData::Str(compact_str::CompactString::const_new(str)),
            ..Self::DEFAULT
        }
    }

    pub fn new_str(str: &str) -> Self {
        Self {
            data: PixelData::Str(compact_str::CompactString::new(str)),
            ..Self::DEFAULT
        }
    }

    pub const fn transparent(mut self) -> Self {
        self.foreground = Color::Transparent;
        self
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
        if self.foreground.is_transparent() {
            self.foreground = other.foreground;
            self.data = other.data;
        }

        if self.background.is_transparent() {
            self.background = other.background
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.merge_mut(other);
        self
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub struct SurfaceRenderer {
    size: Size,

    front: Surface<Pixel>,
    back: Surface<Pixel>,

    default_fg: Color,
    default_bg: Color,
}

impl SurfaceRenderer {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            front: Surface::new(size, Pixel::empty()),
            back: Surface::new(size, Pixel::dirty()),
            default_fg: Color::default(),
            default_bg: Color::default(),
        }
    }

    pub fn default_fg(mut self, default_fg: impl Into<Color>) -> Self {
        self.default_fg = default_fg.into();
        self
    }

    pub fn default_bg(mut self, default_bg: impl Into<Color>) -> Self {
        self.default_bg = default_bg.into();
        self
    }

    pub fn set_default_fg(&mut self, default_fg: impl Into<Color>) {
        self.default_fg = default_fg.into();
    }

    pub fn set_default_bg(&mut self, default_bg: impl Into<Color>) {
        self.default_bg = default_bg.into();
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub fn put(&mut self, pos: Position, pixel: Pixel) {
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x >= self.size.width || y >= self.size.height {
            return;
        }
        let pos = Position::new(x as _, y as _);
        self.front[pos].merge_mut(pixel);
    }

    pub fn patch(&mut self, pos: Position, mut patch: impl FnMut(&mut Pixel)) {
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x >= self.size.width || y >= self.size.height {
            return;
        }

        let pixel = &mut self.front[Position::new(x as _, y as _)];
        patch(pixel)
    }

    pub fn patch_area(
        &mut self,
        pos: Position,
        size: Size,
        mut patch: impl FnMut(Position, &mut Pixel),
    ) {
        // TODO find a less tedious way of doing this
        // maybe size.contains(pos)
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x >= self.size.width || y >= self.size.height {
            return;
        }

        let w = size.width.clamp(0, self.size.width);
        let h = size.height.clamp(0, self.size.height);

        for y in y..h {
            for x in x..w {
                let pos = Position::new(x as _, y as _);
                let pixel = &mut self.front[pos];
                patch(pos, pixel)
            }
        }
    }

    pub fn fill(&mut self, pos: Position, size: Size, pixel: Pixel) {
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x >= self.size.width || y >= self.size.height {
            return;
        }

        let w = size.width.clamp(0, self.size.width);
        let h = size.height.clamp(0, self.size.height);

        for y in y..h {
            for x in x..w {
                self.put(Position::new(x as _, y as _), pixel.clone());
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill(Position::ZERO, self.size, Pixel::default());
    }

    pub fn resize(&mut self, size: Size, mode: ResizeMode) {
        if self.size == size {
            return;
        }
        self.size = size;
        match mode {
            ResizeMode::Keep => {
                self.front.resize(size, mode);
                self.back.resize(size, mode);
            }
            ResizeMode::Discard => {
                self.front.resize(size, mode);
                if self.back.size().height > size.height {
                    self.back.resize(size, mode);
                    self.back.fill(Pixel::dirty());
                } else {
                    self.back.resize(size, ResizeMode::Keep);
                }
            }
        }
    }

    pub fn render<R>(&mut self, rasterizer: &mut R) -> Result<(), R::Error>
    where
        R: Rasterizer,
    {
        #[derive(Default)]
        struct State {
            attr: Option<Attributes>,
            pos: Option<Position>,
            seen: bool,
            fg: Color,
            bg: Color,
            is_reset: bool,
            wrote_attr: bool,
        }
        impl State {
            fn update(&mut self) -> bool {
                let old = self.seen;
                self.seen |= true;
                old
            }

            fn maybe_attr(&mut self, attr: Option<Attributes>) -> Option<Attributes> {
                if attr == attr {
                    return None;
                }
                self.is_reset = false;
                self.wrote_attr = true;
                self.attr = attr;
                attr
            }

            fn reset_attr(&mut self, fg: Color, bg: Color) -> (bool, Option<Color>, Option<Color>) {
                let should_reset = self.wrote_attr;

                self.wrote_attr = false;
                self.is_reset = true;

                let mut bg_set = None;
                if self.bg != bg {
                    bg_set = Some(bg)
                }

                let mut fg_set = None;
                if self.fg != fg {
                    fg_set = Some(fg)
                }

                self.bg = bg;
                self.fg = fg;
                (should_reset, fg_set, bg_set)
            }

            fn maybe_fg(&mut self, fg: Color) -> Option<Color> {
                Self::maybe_color(&mut self.fg, fg)
            }

            fn maybe_bg(&mut self, bg: Color) -> Option<Color> {
                Self::maybe_color(&mut self.bg, bg)
            }

            fn maybe_color(color: &mut Color, new: Color) -> Option<Color> {
                if *color == new {
                    return None;
                }
                *color = new;
                Some(new)
            }

            fn maybe_move(&mut self, pos: Position) -> bool {
                let should_move = match self.pos {
                    // TODO grapheme width
                    Some(old) if old.y != pos.y || old.x != pos.x.saturating_sub(1) => true,
                    None => true,
                    _ => false,
                };
                self.pos = Some(pos);
                should_move
            }
        }

        let mut bytes = [0u8; 4];
        let mut state = State {
            fg: self.default_fg,
            bg: self.default_bg,
            ..State::default()
        };

        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let pos = Position::new(x as i32, y as i32); // FIXME this can lose resolution
                let pixel = std::mem::replace(&mut self.front[pos], Pixel::DEFAULT);
                if pixel == self.back[pos] {
                    continue;
                }

                if !state.update() {
                    rasterizer.begin()?;
                    if state.maybe_move(pos) {
                        rasterizer.move_to(pos)?;
                    }
                    rasterizer.default_fg(self.default_fg)?;
                    rasterizer.default_bg(self.default_bg)?;
                }

                if state.maybe_move(pos) {
                    rasterizer.move_to(pos)?;
                }

                let fg = pixel.foreground.get_or_default(self.default_fg);
                let bg = pixel.background.get_or_default(self.default_bg);

                let attr = pixel.attributes;
                match state.maybe_attr(attr) {
                    Some(attr) => rasterizer.set_attribute(attr)?,
                    _ if attr.is_none() => {
                        let (reset, fg, bg) = state.reset_attr(fg, bg);
                        if reset {
                            rasterizer.reset_attribute()?;
                        }
                        if let Some(bg) = bg {
                            rasterizer.set_bg(bg)?;
                        }
                        if let Some(fg) = fg {
                            rasterizer.set_fg(fg)?;
                        }
                    }
                    _ => {}
                }

                if let Some(fg) = state.maybe_fg(fg) {
                    rasterizer.set_fg(fg)?;
                }
                if let Some(bg) = state.maybe_bg(bg) {
                    rasterizer.set_bg(bg)?;
                }

                let s = match &pixel.data {
                    PixelData::Char(ch) => ch.encode_utf8(&mut bytes),
                    PixelData::Str(s) => &**s,
                };
                rasterizer.write(s)?;
                self.back[pos] = pixel;
            }
        }

        if state.seen {
            rasterizer.end()?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Surface<T> {
    pos: Position,
    size: Size,
    default: T,
    pixels: Vec<T>,
}

impl<T> Surface<T> {
    pub fn new(size: Size, default: T) -> Self
    where
        T: Clone,
    {
        Self {
            pos: Position::ZERO,
            size,
            default: default.clone(),
            pixels: vec![default; size.area() as usize],
        }
    }

    pub const fn with_offset(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }

    pub fn clear(&mut self)
    where
        T: Clone,
    {
        self.fill(self.default.clone());
    }

    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.pixels.fill(value);
    }

    pub fn resize(&mut self, size: Size, mode: ResizeMode)
    where
        T: Clone,
    {
        let area = size.area() as usize;
        match mode {
            ResizeMode::Keep => {
                let mut new = vec![self.default.clone(); area];
                let min = self.size.min(size);
                for y in 0..min.height {
                    let (x0, x1) = (
                        y as usize * self.size.width as usize,
                        y as usize * min.width as usize,
                    );
                    let (y0, y1) = (x0 + min.width as usize, x1 + min.width as usize);
                    new[x1..y1].clone_from_slice(&self.pixels[x0..y0]);
                }
                self.pixels = new;
            }
            ResizeMode::Discard => {
                self.pixels.resize(area, self.default.clone());
                self.pixels.fill(self.default.clone());
            }
        }
        self.size = size;
    }

    const fn pos_of(stride: u32, pos: Position) -> u32 {
        pos.y as u32 * stride + pos.x as u32
    }

    pub const fn position(&self) -> Position {
        self.pos
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    #[inline(always)]
    pub fn get(&self, pos: Position) -> Option<&T> {
        let index = Self::pos_of(self.size.width, pos + self.pos) as usize;
        self.pixels.get(index)
    }

    #[inline(always)]
    pub fn get_mut(&mut self, pos: Position) -> Option<&mut T> {
        let index = Self::pos_of(self.size.width, pos + self.pos) as usize;
        self.pixels.get_mut(index)
    }

    pub fn set(&mut self, pos: Position, value: T) {
        let Some(pixel) = self.get_mut(pos) else {
            return;
        };
        *pixel = value
    }

    pub fn copy_row(&mut self, pos: Position, row: &[T])
    where
        T: Copy,
    {
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x >= self.size.width || y >= self.size.height {
            return;
        }

        let start = Self::pos_of(self.size.width, pos) as usize;
        if start >= self.pixels.len() {
            return;
        }

        let w = self.size.width as usize;
        let stride = w.min(row.len());
        let len = self.pixels.len();
        let end = (start + stride).min(len);

        self.pixels[start..end].copy_from_slice(&row[..stride.min(end - start)]);
    }

    pub fn clone_row(&mut self, pos: Position, row: &[T])
    where
        T: Clone,
    {
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x >= self.size.width || y >= self.size.height {
            return;
        }

        let start = Self::pos_of(self.size.width, pos) as usize;
        if start >= self.pixels.len() {
            return;
        }

        let w = self.size.width as usize;
        let stride = w.min(row.len());
        let len = self.pixels.len();
        let end = (start + stride).min(len);

        self.pixels[start..end].clone_from_slice(&row[..stride.min(end - start)]);
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (Position, &T)> + DoubleEndedIterator {
        self.pixels.iter().enumerate().map(|(i, p)| {
            let x = i as u32 % self.size.width;
            let y = i as u32 / self.size.width;
            (pos(x as _, y as _), p)
        })
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (Position, &mut T)> + DoubleEndedIterator {
        self.pixels.iter_mut().enumerate().map(|(i, p)| {
            let x = i as u32 % self.size.width;
            let y = i as u32 / self.size.width;
            (pos(x as _, y as _), p)
        })
    }

    pub fn rows(&self) -> impl ExactSizeIterator<Item = (u32, &[T])> + DoubleEndedIterator {
        self.pixels
            .chunks_exact(self.size.width as usize)
            .enumerate()
            .map(|(i, c)| (i as u32, c))
    }

    pub fn rows_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (u32, &mut [T])> + DoubleEndedIterator {
        self.pixels
            .chunks_exact_mut(self.size.width as usize)
            .enumerate()
            .map(|(i, c)| (i as u32, c))
    }
}

impl<T> std::ops::Index<Position> for Surface<T> {
    type Output = T;
    #[track_caller]
    #[inline]
    fn index(&self, index: Position) -> &Self::Output {
        let index = Self::pos_of(self.size.width, index + self.pos) as usize;
        &self.pixels[index]
    }
}

impl<T> std::ops::IndexMut<Position> for Surface<T> {
    #[track_caller]
    #[inline]
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        let index = Self::pos_of(self.size.width, index + self.pos) as usize;
        &mut self.pixels[index]
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum ResizeMode {
    Keep,
    #[default]
    Discard,
}

pub trait Rasterizer {
    type Error: std::error::Error;

    fn begin(&mut self) -> Result<(), Self::Error>;
    fn end(&mut self) -> Result<(), Self::Error>;

    fn clear(&mut self, pos: Position, size: Size) -> Result<(), Self::Error>;

    fn move_to(&mut self, pos: Position) -> Result<(), Self::Error>;

    fn default_fg(&mut self, color: Color) -> Result<(), Self::Error>;
    fn default_bg(&mut self, color: Color) -> Result<(), Self::Error>;

    fn set_fg(&mut self, color: Color) -> Result<(), Self::Error>;
    fn set_bg(&mut self, color: Color) -> Result<(), Self::Error>;

    fn set_attribute(&mut self, attribute: Attributes) -> Result<(), Self::Error>;

    // fn set_underline(&mut self) -> Result<(), Self::Error>;
    // fn set_underline_color(&mut self) -> Result<(), Self::Error>;

    fn reset_fg(&mut self) -> Result<(), Self::Error>;
    fn reset_bg(&mut self) -> Result<(), Self::Error>;
    fn reset_attribute(&mut self) -> Result<(), Self::Error>;
    // fn reset_underline_color(&mut self) -> Result<(), Self::Error>;

    fn write(&mut self, data: &str) -> Result<(), Self::Error>;
}

impl<T> Rasterizer for &mut T
where
    T: Rasterizer,
{
    type Error = T::Error;

    #[inline(always)]
    fn begin(&mut self) -> Result<(), Self::Error> {
        (**self).begin()
    }

    #[inline(always)]
    fn end(&mut self) -> Result<(), Self::Error> {
        (**self).end()
    }

    #[inline(always)]
    fn clear(&mut self, pos: Position, size: Size) -> Result<(), Self::Error> {
        (**self).clear(pos, size)
    }

    #[inline(always)]
    fn move_to(&mut self, pos: Position) -> Result<(), Self::Error> {
        (**self).move_to(pos)
    }

    #[inline(always)]
    fn default_fg(&mut self, color: Color) -> Result<(), Self::Error> {
        (**self).default_fg(color)
    }

    #[inline(always)]
    fn default_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        (**self).default_bg(color)
    }

    #[inline(always)]
    fn set_fg(&mut self, color: Color) -> Result<(), Self::Error> {
        (**self).set_fg(color)
    }

    #[inline(always)]
    fn set_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        (**self).set_bg(color)
    }

    #[inline(always)]
    fn set_attribute(&mut self, attribute: Attributes) -> Result<(), Self::Error> {
        (**self).set_attribute(attribute)
    }

    #[inline(always)]
    fn reset_fg(&mut self) -> Result<(), Self::Error> {
        (**self).reset_fg()
    }

    #[inline(always)]
    fn reset_bg(&mut self) -> Result<(), Self::Error> {
        (**self).reset_bg()
    }

    #[inline(always)]
    fn reset_attribute(&mut self) -> Result<(), Self::Error> {
        (**self).reset_attribute()
    }

    #[inline(always)]
    fn write(&mut self, data: &str) -> Result<(), Self::Error> {
        (**self).write(data)
    }
}

macro_rules! csi {
    ($($lit:literal),*) => {
        concat!($("\x1b[",$lit),*).as_bytes()
    };
}

#[derive(Default)]
pub struct BufferedRasterizer {
    inner: Cursor<Vec<u8>>,
}

impl BufferedRasterizer {
    pub const fn new() -> Self {
        Self::with_cursor(Cursor::new(Vec::new()))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_cursor(Cursor::new(Vec::with_capacity(capacity)))
    }

    const fn with_cursor(cursor: Cursor<Vec<u8>>) -> Self {
        Self { inner: cursor }
    }

    pub fn copy_to(&mut self, mut out: impl std::io::Write) -> std::io::Result<()> {
        let pos = self.inner.position() as usize;
        self.inner.set_position(0);
        if pos == 0 {
            return Ok(());
        }

        let mut input = &self.inner.get_ref()[..pos];
        std::io::copy(&mut input, &mut out)?;
        out.flush()
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), <Self as Rasterizer>::Error> {
        _ = self.inner.write_all(data);
        Ok(())
    }

    fn write_fmt(&mut self, f: std::fmt::Arguments<'_>) -> Result<(), <Self as Rasterizer>::Error> {
        _ = write!(&mut self.inner, "{f}");
        Ok(())
    }
}

impl std::io::Write for BufferedRasterizer {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Rasterizer for BufferedRasterizer {
    type Error = Infallible;

    fn begin(&mut self) -> Result<(), Self::Error> {
        self.write_bytes(csi!("?2026h"))
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        self.write_bytes(csi!("?2026l"))?;
        _ = self.inner.flush();
        Ok(())
    }

    fn clear(&mut self, pos: Position, size: Size) -> Result<(), Self::Error> {
        self.write_fmt(format_args!(
            "\x1b[{y};{x};{h};{w}$z",
            y = pos.y + 1,
            x = pos.x + 1,
            h = size.height,
            w = size.width
        ))
    }

    fn move_to(&mut self, pos: Position) -> Result<(), Self::Error> {
        self.write_fmt(format_args!("\x1b[{y};{x};H", y = pos.y + 1, x = pos.x + 1,))
    }

    fn default_fg(&mut self, color: Color) -> Result<(), Self::Error> {
        self.set_fg(color)
    }

    fn default_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        self.set_bg(color)
    }

    fn set_fg(&mut self, color: Color) -> Result<(), Self::Error> {
        match color {
            Color::Named(IndexedColor(color)) => self.write_fmt(format_args!("\x1b[38;5;{color}m")),
            Color::Rgba(Rgba(r, g, b, _)) => self.write_fmt(format_args!("\x1b[38;2;{r};{g};{b}m")),
            Color::Default => self.reset_fg(),
            Color::Transparent => Ok(()),
        }
    }

    fn set_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        match color {
            Color::Named(IndexedColor(color)) => self.write_fmt(format_args!("\x1b[48;5;{color}m")),
            Color::Rgba(Rgba(r, g, b, _)) => self.write_fmt(format_args!("\x1b[48;2;{r};{g};{b}m")),
            Color::Default => self.reset_bg(),
            Color::Transparent => Ok(()),
        }
    }

    fn set_attribute(&mut self, attribute: Attributes) -> Result<(), Self::Error> {
        let mut seen = false;
        for i in attribute.as_indexed_bytes() {
            seen = true;
            self.write_fmt(format_args!("\x1b[{i}m"))?;
        }
        if !seen {
            self.write_bytes(csi!("0m"))?;
        }
        Ok(())
    }

    fn reset_fg(&mut self) -> Result<(), Self::Error> {
        self.write_bytes(csi!("39m"))
    }

    fn reset_bg(&mut self) -> Result<(), Self::Error> {
        self.write_bytes(csi!("49m"))
    }

    fn reset_attribute(&mut self) -> Result<(), Self::Error> {
        self.write_bytes(csi!("59m"))
    }

    fn write(&mut self, data: &str) -> Result<(), Self::Error> {
        self.write_fmt(format_args!("{data}"))
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rgba(u8, u8, u8, u8);

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }

    pub const fn from_u32(color: u32) -> Self {
        let a = (color >> 12) & ((1 << 4) - 1);
        let is_16 = a == 0;
        let offset = if is_16 { 4 } else { 0 };
        let r = ((color >> (12 - offset)) & 0xF) as u8;
        let g = ((color >> (8 - offset)) & 0xF) as u8;
        let b = ((color >> (4 - offset)) & 0xF) as u8;
        let a = if is_16 { 0xF } else { (color & 0xF) as u8 };
        Self((r << 4) | r, (g << 4) | g, (b << 4) | b, (a << 4) | a)
    }

    pub fn from_float([r, g, b, a]: [f32; 4]) -> Self {
        let scale = |d: f32| (255.0_f32 * d.clamp(0.0, 1.0)).round() as u8;
        Self(scale(r), scale(g), scale(b), scale(a))
    }

    pub const fn hex(color: &str) -> Self {
        todo!();
    }

    pub fn to_hsva(self) -> Hsva {
        todo!();
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hsva(f32, f32, f32, f32);

impl Hsva {
    pub fn to_rgba(self) -> Rgba {
        todo!();
    }
}

// should we have a special 'transparent' color?
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum Color {
    Named(IndexedColor),
    Rgba(Rgba),
    Transparent,
    #[default]
    Default,
}

impl Color {
    pub const fn is_transparent(&self) -> bool {
        matches!(self, Self::Transparent)
    }

    pub fn get_or_default(self, other: impl Into<Self>) -> Self {
        match self {
            Self::Transparent | Self::Default => other.into(),
            this => this,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IndexedColor(pub u8);

impl IndexedColor {
    pub const fn new(color: u8) -> Self {
        Self(color)
    }

    pub const fn approximate_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(color_helpers::rgb_to_ansi(r, g, b))
    }
    pub const fn black() -> Self {
        Self(0)
    }

    pub const fn white() -> Self {
        Self(15)
    }

    pub const fn grey() -> Self {
        Self(8)
    }

    pub const fn light_grey() -> Self {
        Self(7)
    }

    pub const fn red() -> Self {
        Self(1)
    }

    pub const fn light_red() -> Self {
        Self(9)
    }

    pub const fn green() -> Self {
        Self(2)
    }

    pub const fn light_green() -> Self {
        Self(10)
    }

    pub const fn yellow() -> Self {
        Self(3)
    }

    pub const fn light_yellow() -> Self {
        Self(11)
    }

    pub const fn blue() -> Self {
        Self(4)
    }

    pub const fn light_blue() -> Self {
        Self(12)
    }

    pub const fn magenta() -> Self {
        Self(5)
    }

    pub const fn light_magenta() -> Self {
        Self(13)
    }

    pub const fn cyan() -> Self {
        Self(6)
    }

    pub const fn light_cyan() -> Self {
        Self(14)
    }

    pub const fn to_3bit(self) -> u8 {
        self.to_4bit() % 8
    }

    pub const fn to_4bit(self) -> u8 {
        color_helpers::ansi_to_4bit(self.0)
    }

    pub const fn to_8bit(self) -> u8 {
        self.0
    }

    pub const fn to_rgb(self) -> Rgba {
        let (r, g, b) = color_helpers::ansi_to_rgb(self.0);
        Rgba(r, g, b, 0xFF)
    }
}

impl From<IndexedColor> for Color {
    fn from(value: IndexedColor) -> Self {
        Self::Named(value)
    }
}

impl From<Rgba> for Color {
    fn from(value: Rgba) -> Self {
        Self::Rgba(value)
    }
}

impl<T> From<Option<T>> for Color
where
    T: Into<Self>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(s) => s.into(),
            None => Self::default(),
        }
    }
}

mod color_helpers {
    pub const fn rgb_to_4bit(r: u8, g: u8, b: u8) -> u8 {
        const fn extract(c: u8) -> (u8, u8) {
            match c {
                230.. => (1, 1),
                180.. => (1, 0),
                80.. => (0, 1),
                _ => (0, 0),
            }
        }

        let (hr, r) = extract(r);
        let (hg, g) = extract(g);
        let (hb, b) = extract(b);

        let v = r | hr | (g << 1) | (hg << 1) | (b << 2) | (hb << 2) | ((hr | hg | hb) << 3);
        match v {
            0b1111 if r & g & b == 0 => 7,
            0b0111 => 8,
            0b0000..=0b1111 => v,
            _ => 15,
        }
    }

    pub const fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
        let c = rgb_to_4bit(r, g, b);
        let (pr, pg, pb) = ansi_to_rgb(c);
        if r == pr && g == pg && b == pb {
            c
        } else if r == g && g == b {
            color256_to_ansi_grey(r)
        } else {
            rgb_to_ansi_index(r, g, b)
        }
    }

    pub const fn ansi_to_4bit(ansi: u8) -> u8 {
        if ansi < 16 {
            return ansi;
        }
        let (r, g, b) = ansi_to_rgb(ansi);
        rgb_to_4bit(r, g, b)
    }

    pub const fn ansi_to_rgb(ansi: u8) -> (u8, u8, u8) {
        let c = COLOR_256[ansi as usize];
        ((c >> 16) as u8, (c >> 8) as u8, c as u8)
    }

    const fn color256_to_ansi_grey(c: u8) -> u8 {
        match 232u8.checked_add(c / 10) {
            Some(c) => c,
            None => 255,
        }
    }

    const fn rgb_to_ansi_index(r: u8, g: u8, b: u8) -> u8 {
        16 + 36 * (r / 51) + 6 * (g / 51) + (b / 51)
    }

    pub const COLOR_256: [u32; 256] = [
        0x000000, 0x800000, 0x008000, 0x808000, 0x0000EE, 0x800080, 0x008080, 0xC0C0C0, //
        0x808080, 0xFF6600, 0x00FF00, 0xFFFF00, 0x6699FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF, //
        0x000000, 0x00005F, 0x000087, 0x0000AF, 0x0000D7, 0x0000FF, 0x005F00, 0x005F5F, //
        0x005F87, 0x005FAF, 0x005FD7, 0x005FFF, 0x008700, 0x00875F, 0x008787, 0x0087AF, //
        0x0087D7, 0x0087FF, 0x00AF00, 0x00AF5F, 0x00AF87, 0x00AFAF, 0x00AFD7, 0x00AFFF, //
        0x00D700, 0x00D75F, 0x00D787, 0x00D7AF, 0x00D7D7, 0x00D7FF, 0x00FF00, 0x00FF5F, //
        0x00FF87, 0x00FFAF, 0x00FFD7, 0x00FFFF, 0x5F0000, 0x5F005F, 0x5F0087, 0x5F00AF, //
        0x5F00D7, 0x5F00FF, 0x5F5F00, 0x5F5F5F, 0x5F5F87, 0x5F5FAF, 0x5F5FD7, 0x5F5FFF, //
        0x5F8700, 0x5F875F, 0x5F8787, 0x5F87AF, 0x5F87D7, 0x5F87FF, 0x5FAF00, 0x5FAF5F, //
        0x5FAF87, 0x5FAFAF, 0x5FAFD7, 0x5FAFFF, 0x5FD700, 0x5FD75F, 0x5FD787, 0x5FD7AF, //
        0x5FD7D7, 0x5FD7FF, 0x5FFF00, 0x5FFF5F, 0x5FFF87, 0x5FFFAF, 0x5FFFD7, 0x5FFFFF, //
        0x870000, 0x87005F, 0x870087, 0x8700AF, 0x8700D7, 0x8700FF, 0x875F00, 0x875F5F, //
        0x875F87, 0x875FAF, 0x875FD7, 0x875FFF, 0x878700, 0x87875F, 0x878787, 0x8787AF, //
        0x8787D7, 0x8787FF, 0x87AF00, 0x87AF5F, 0x87AF87, 0x87AFAF, 0x87AFD7, 0x87AFFF, //
        0x87D700, 0x87D75F, 0x87D787, 0x87D7AF, 0x87D7D7, 0x87D7FF, 0x87FF00, 0x87FF5F, //
        0x87FF87, 0x87FFAF, 0x87FFD7, 0x87FFFF, 0xAF0000, 0xAF005F, 0xAF0087, 0xAF00AF, //
        0xAF00D7, 0xAF00FF, 0xAF5F00, 0xAF5F5F, 0xAF5F87, 0xAF5FAF, 0xAF5FD7, 0xAF5FFF, //
        0xAF8700, 0xAF875F, 0xAF8787, 0xAF87AF, 0xAF87D7, 0xAF87FF, 0xAFAF00, 0xAFAF5F, //
        0xAFAF87, 0xAFAFAF, 0xAFAFD7, 0xAFAFFF, 0xAFD700, 0xAFD75F, 0xAFD787, 0xAFD7AF, //
        0xAFD7D7, 0xAFD7FF, 0xAFFF00, 0xAFFF5F, 0xAFFF87, 0xAFFFAF, 0xAFFFD7, 0xAFFFFF, //
        0xD70000, 0xD7005F, 0xD70087, 0xD700AF, 0xD700D7, 0xD700FF, 0xD75F00, 0xD75F5F, //
        0xD75F87, 0xD75FAF, 0xD75FD7, 0xD75FFF, 0xD78700, 0xD7875F, 0xD78787, 0xD787AF, //
        0xD787D7, 0xD787FF, 0xD7AF00, 0xD7AF5F, 0xD7AF87, 0xD7AFAF, 0xD7AFD7, 0xD7AFFF, //
        0xD7D700, 0xD7D75F, 0xD7D787, 0xD7D7AF, 0xD7D7D7, 0xD7D7FF, 0xD7FF00, 0xD7FF5F, //
        0xD7FF87, 0xD7FFAF, 0xD7FFD7, 0xD7FFFF, 0xFF0000, 0xFF005F, 0xFF0087, 0xFF00AF, //
        0xFF00D7, 0xFF00FF, 0xFF5F00, 0xFF5F5F, 0xFF5F87, 0xFF5FAF, 0xFF5FD7, 0xFF5FFF, //
        0xFF8700, 0xFF875F, 0xFF8787, 0xFF87AF, 0xFF87D7, 0xFF87FF, 0xFFAF00, 0xFFAF5F, //
        0xFFAF87, 0xFFAFAF, 0xFFAFD7, 0xFFAFFF, 0xFFD700, 0xFFD75F, 0xFFD787, 0xFFD7AF, //
        0xFFD7D7, 0xFFD7FF, 0xFFFF00, 0xFFFF5F, 0xFFFF87, 0xFFFFAF, 0xFFFFD7, 0xFFFFFF, //
        0x080808, 0x121212, 0x1C1C1C, 0x262626, 0x303030, 0x3A3A3A, 0x444444, 0x4E4E4E, //
        0x585858, 0x626262, 0x6C6C6C, 0x767676, 0x808080, 0x8A8A8A, 0x949494, 0x9E9E9E, //
        0xA8A8A8, 0xB2B2B2, 0xBCBCBC, 0xC6C6C6, 0xD0D0D0, 0xDADADA, 0xE4E4E4, 0xEEEEEE, //
    ];
}
