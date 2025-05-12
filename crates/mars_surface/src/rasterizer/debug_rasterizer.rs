use std::{convert::Infallible, fmt::Write as _};

use mars_math::{Position, Size};

use crate::{Attributes, Color, IndexedColor, Rasterizer, Rgba};

#[derive(Debug)]
pub struct DebugRasterizer {
    out: String,
    incomplete: bool,
}

impl Default for DebugRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugRasterizer {
    pub const fn new() -> Self {
        Self {
            out: String::new(),
            incomplete: false,
        }
    }

    fn next_entry(&mut self) {
        if self.incomplete {
            self.out.push('\n');
            self.incomplete = !self.incomplete
        }
    }

    fn set_color(
        &mut self,
        name: &str,
        color: Color,
        or: impl Fn(&mut Self) -> Result<(), <Self as Rasterizer>::Error>,
    ) -> Result<(), <Self as Rasterizer>::Error> {
        match color {
            Color::Named(IndexedColor(index)) => {
                _ = writeln!(&mut self.out, "    {name}: {index}");
            }
            Color::Rgba(Rgba(r, g, b, a)) => {
                _ = writeln!(&mut self.out, "    {name}: #{r:02X}{g:02X}{b:02X}{a:02X}");
            }
            Color::Default => or(self)?,
        }

        Ok(())
    }
}

impl std::fmt::Display for DebugRasterizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out.fmt(f)
    }
}

impl Rasterizer for DebugRasterizer {
    type Error = Infallible;

    fn begin(&mut self) -> Result<(), Self::Error> {
        self.out.clear();
        self.incomplete = false;
        self.next_entry();
        _ = writeln!(&mut self.out, "begin");
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        self.next_entry();
        _ = writeln!(&mut self.out, "end");
        Ok(())
    }

    fn clear(&mut self, pos: Position, size: Size) -> Result<(), Self::Error> {
        self.next_entry();
        _ = writeln!(
            &mut self.out,
            "    clear {x},{y} .. {w},{h}",
            x = pos.x,
            y = pos.y,
            w = size.width,
            h = size.height
        );
        Ok(())
    }

    fn clear_screen(&mut self, bg: Color, _size: Size) -> Result<(), Self::Error> {
        self.next_entry();
        self.set_color("clear_screen", bg, |_| Ok(()))
    }

    fn move_to(&mut self, pos: Position) -> Result<(), Self::Error> {
        self.next_entry();
        _ = writeln!(&mut self.out, "    move to: {x},{y}", x = pos.x, y = pos.y);
        Ok(())
    }

    fn default_fg(&mut self, color: Color) -> Result<(), Self::Error> {
        self.next_entry();
        self.set_color("set_default_fg", color, |_| Ok(()))
    }

    fn default_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        self.next_entry();
        self.set_color("set_default_bg", color, |_| Ok(()))
    }

    fn set_fg(&mut self, color: Color) -> Result<(), Self::Error> {
        self.next_entry();
        self.set_color("set_fg", color, |_| Ok(()))
    }

    fn set_bg(&mut self, color: Color) -> Result<(), Self::Error> {
        self.next_entry();
        self.set_color("set_bg", color, |_| Ok(()))
    }

    fn set_attribute(&mut self, attribute: Attributes) -> Result<(), Self::Error> {
        self.next_entry();
        for (i, ul) in attribute.as_indexed_bytes().enumerate() {
            if i > 0 {
                _ = write!(&mut self.out, " | ");
            }
            _ = write!(&mut self.out, "{ul}");
        }
        _ = writeln!(&mut self.out);
        Ok(())
    }

    fn reset_fg(&mut self) -> Result<(), Self::Error> {
        self.next_entry();
        _ = writeln!(&mut self.out, "   reset_fg");
        Ok(())
    }

    fn reset_bg(&mut self) -> Result<(), Self::Error> {
        self.next_entry();
        _ = writeln!(&mut self.out, "   reset_bg");
        Ok(())
    }

    fn reset_attribute(&mut self) -> Result<(), Self::Error> {
        self.next_entry();
        _ = writeln!(&mut self.out, "   reset_attribute");
        Ok(())
    }

    fn write(&mut self, data: &str) -> Result<(), Self::Error> {
        if !std::mem::replace(&mut self.incomplete, true) {
            self.out.push_str("    ");
        }

        // TODO grapheme clusters
        for cluster in data.chars() {
            let cluster = if cluster.is_whitespace() {
                '▪'
            } else {
                cluster
            };
            _ = write!(&mut self.out, "{cluster}")
        }

        Ok(())
    }
}
