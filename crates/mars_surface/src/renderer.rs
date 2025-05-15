use std::ops::RangeInclusive;

mod basic_renderer;
pub use basic_renderer::BasicRenderer;

use crate::{Color, Drawable, Pixel, Rasterizer};
use mars_math::{Axis, Position, Size};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum BlendMode {
    Blend,
    #[default]
    Replace,
}

pub trait Renderer: RendererSetup + Placer {
    fn get(&self, pos: Position) -> Option<&Pixel>;
    fn get_mut(&mut self, pos: Position) -> Option<&mut Pixel>;

    fn patch(&mut self, pos: Position, mut patch: impl FnMut(&mut Pixel)) {
        if let Some(pixel) = self.get_mut(pos) {
            patch(pixel)
        }
    }

    fn patch_area(
        &mut self,
        pos: Position,
        size: Size,
        mut patch: impl FnMut(Position, &mut Pixel),
    ) {
        let Some(pos) = pos.to_unsigned_checked() else {
            return;
        };

        let our_size = self.size();
        let w = size.width.clamp(0, our_size.width);
        let h = size.height.clamp(0, our_size.height);

        if pos.x > w || pos.y >= h {
            return;
        }

        for y in pos.y..h {
            for x in pos.x..w {
                let pos = Position::new(x, y).to_signed();
                self.patch(pos, |pixel| patch(pos, pixel));
            }
        }
    }

    fn clear(&mut self) {
        self.fill(
            Position::ZERO,
            self.size(),
            Pixel::default(),
            BlendMode::Replace,
        );
    }

    fn fill(&mut self, pos: Position, size: Size, pixel: Pixel, blend: BlendMode) {
        let Some(pos) = pos.to_unsigned_checked() else {
            return;
        };

        let our_size = self.size();
        let w = size.width.clamp(0, our_size.width);
        let h = size.height.clamp(0, our_size.height);

        for y in pos.y..h {
            for x in pos.x..w {
                let pos = Position::new(x, y).to_signed();
                self.put(pos, pixel.clone(), blend);
            }
        }
    }

    fn fill_with(
        &mut self,
        origin: Position<u32>,
        size: Size,
        mut with: impl FnMut(Position<u32>) -> Pixel,
    ) {
        for y in origin.y..size.height {
            for x in origin.x..size.width {
                let pos = Position::new(x, y);
                self.patch(pos.to_signed(), |pixel| *pixel = with(pos));
            }
        }
    }

    // does the blend mode really matter?
    fn draw(&mut self, render: impl Drawable, blend: BlendMode)
    where
        Self: Sized,
    {
        render.draw(self, Position::ZERO, blend);
    }

    fn render<R: Rasterizer>(&mut self, rasterizer: R) -> Result<(), R::Error>;
}

pub trait RendererSetup {
    fn default_colors(&self) -> (Color, Color) {
        (Color::default(), Color::default())
    }
    fn set_default_fg(&mut self, default_fg: impl Into<Color>) {
        _ = default_fg
    }
    fn set_default_bg(&mut self, default_bg: impl Into<Color>) {
        _ = default_bg
    }
}

pub trait Placer {
    fn put(&mut self, pos: Position, pixel: Pixel, blend: BlendMode);
    fn size(&self) -> Size;
}

pub trait PlacerExt<'a: 'b, 'b>: Placer + 'a {
    fn set(&'b mut self, pos: Position, pixel: Pixel, blend: BlendMode) -> &'b mut Self {
        self.put(pos, pixel, blend);
        self
    }

    fn horizontal_line(
        &'b mut self,
        start: Position,
        range: RangeInclusive<i32>,
        pixel: Pixel,
        blend: BlendMode,
    ) -> &'b mut Self {
        self.line(Axis::Horizontal, start, range, pixel, blend)
    }

    fn vertical_line(
        &'b mut self,
        start: Position,
        range: RangeInclusive<i32>,
        pixel: Pixel,
        blend: BlendMode,
    ) -> &'b mut Self {
        self.line(Axis::Vertical, start, range, pixel, blend)
    }

    fn line(
        &'b mut self,
        axis: Axis,
        offset: Position,
        range: RangeInclusive<i32>,
        pixel: Pixel,
        blend: BlendMode,
    ) -> &'b mut Self {
        let cross: i32 = axis.cross(offset);
        let start: Position = axis.pack(*range.start(), cross);
        let end: Position = axis.pack(*range.end(), cross);
        // FIXME: start can be after end
        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.put(Position::new(x, y), pixel.clone(), blend);
            }
        }
        self
    }
}

impl<'a: 'b, 'b, T: Placer + 'a> PlacerExt<'a, 'b> for T {}
impl<'a: 'b, 'b> PlacerExt<'a, 'b> for dyn Placer + 'a {}
