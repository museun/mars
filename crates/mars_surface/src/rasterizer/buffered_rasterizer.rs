use std::{
    convert::Infallible,
    io::{Cursor, Write as _},
};

use mars_math::{Position, Size};

use crate::{Attributes, Color, IndexedColor, Rasterizer, Rgba};

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

    const fn with_cursor(inner: Cursor<Vec<u8>>) -> Self {
        Self { inner }
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

    fn clear_screen(&mut self, bg: Color, size: Size) -> Result<(), Self::Error> {
        for y in 0..size.height {
            self.move_to(Position::new(0, y as _))?;
            self.set_bg(bg)?;
            self.write_fmt(format_args!("\x1b[{width}@", width = size.width))?
        }
        Ok(())
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
            // BUG this shouldn't reset the fg
            Color::Default => self.reset_fg(),
        }
    }

    fn set_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        match color {
            Color::Named(IndexedColor(color)) => self.write_fmt(format_args!("\x1b[48;5;{color}m")),
            Color::Rgba(Rgba(r, g, b, _)) => self.write_fmt(format_args!("\x1b[48;2;{r};{g};{b}m")),
            // BUG this shouldn't reset the bg
            Color::Default => self.reset_bg(),
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
