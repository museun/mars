use mars_math::{Position, Size};

use crate::{Attributes, Color};

// TODO unsigned positions
pub trait Rasterizer {
    type Error: std::error::Error;

    fn begin(&mut self) -> Result<(), Self::Error>;
    fn end(&mut self) -> Result<(), Self::Error>;

    fn clear(&mut self, pos: Position, size: Size) -> Result<(), Self::Error>;
    fn clear_screen(&mut self, bg: Color, size: Size) -> Result<(), Self::Error>;

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
    fn clear_screen(&mut self, bg: Color, size: Size) -> Result<(), Self::Error> {
        (**self).clear_screen(bg, size)
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

mod buffered_rasterizer;
pub use buffered_rasterizer::BufferedRasterizer;

mod debug_rasterizer;
pub use debug_rasterizer::DebugRasterizer;
