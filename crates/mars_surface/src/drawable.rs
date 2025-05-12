use std::borrow::Cow;

use mars_math::{Anchor2, Position, Size};

use crate::{BlendMode, Color, Pixel, Placer, Renderer, Rgba, pixel::PixelData};

pub trait Drawable {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode);
    fn size(&self, input: Size) -> Size;

    fn render(&self, renderer: &mut impl Renderer, blend: BlendMode)
    where
        for<'a> &'a Self: Drawable,
        Self: Sized,
    {
        renderer.draw(self, blend)
    }
}

impl<T> Drawable for &T
where
    T: Drawable,
{
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        (*self).draw(placer, pos, blend);
    }

    fn size(&self, input: Size) -> Size {
        (*self).size(input)
    }
}

impl Drawable for Pixel {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        placer.put(pos, self.clone(), blend);
    }

    fn size(&self, input: Size) -> Size {
        match &self.data {
            PixelData::Char(c) => c.size(input),
            PixelData::Str(s) => s.as_str().size(input),
        }
    }
}

impl Drawable for Rgba {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        placer.put(pos, Pixel::empty().bg(*self), blend);
    }

    fn size(&self, _: Size) -> Size {
        Size::ZERO
    }
}

impl Drawable for char {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        placer.put(pos, Pixel::new(*self), blend);
    }

    fn size(&self, _: Size) -> Size {
        Size::new(1, 1) // TODO wcswidth
    }
}

impl Drawable for String {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        self.as_str().draw(placer, pos, blend);
    }

    fn size(&self, input: Size) -> Size {
        self.as_str().size(input)
    }
}

impl Drawable for Cow<'_, str> {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        self.as_ref().draw(placer, pos, blend);
    }

    fn size(&self, input: Size) -> Size {
        self.as_ref().size(input)
    }
}

fn measure_text(s: &str, size: Size, mut place: impl FnMut(Position, char)) -> Size {
    if s.is_empty() {
        return Size::ZERO;
    }

    let mut dx = 0;
    let mut dy = 0;
    let mut w = 0;

    // TODO grapheme clusters
    for (_, c) in s.char_indices() {
        if w > size.width {
            break;
        }

        if c == '\n' {
            if dy + 1 > size.width {
                break;
            }
            dy += 1;
            dx = 0;
            continue;
        }

        (place)(Position::new(dx, dy as _), c);
        dx += 1;
        w = w.max(dx as _)
    }

    Size::new(w, dy + 1) // add 1 so we are inclusive
}

impl Drawable for &str {
    fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
        _ = measure_text(self, placer.size(), |p, c| {
            placer.put(p + pos, Pixel::new(c), blend);
        });
    }

    fn size(&self, input: Size) -> Size {
        measure_text(self, input, |_, _| {})
    }
}

impl Drawable for () {
    fn draw(&self, _renderer: &mut dyn Placer, _pos: Position, _blend: BlendMode) {}
    fn size(&self, _: Size) -> Size {
        Size::ZERO
    }
}

pub trait DrawableExt: Drawable {
    fn with_fg(&self, fg: impl Into<Color>) -> impl Drawable
    where
        Self: Sized,
    {
        struct WithFg<'a, R: Drawable> {
            fg: Color,
            drawable: &'a R,
        }
        impl<R: Drawable> Drawable for WithFg<'_, R> {
            fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
                let mut adapter = Adapter {
                    placer,
                    fg: self.fg,
                };
                self.drawable.draw(&mut adapter, pos, blend)
            }

            fn size(&self, input: Size) -> Size {
                self.drawable.size(input)
            }
        }

        struct Adapter<'a> {
            placer: &'a mut dyn Placer,
            fg: Color,
        }

        impl Placer for Adapter<'_> {
            fn put(&mut self, pos: Position, pixel: Pixel, blend: BlendMode) {
                self.placer.put(pos, pixel.fg(self.fg), blend);
            }
            fn size(&self) -> Size {
                self.placer.size()
            }
        }

        WithFg {
            fg: fg.into(),
            drawable: self,
        }
    }

    fn with_bg(&self, bg: impl Into<Color>) -> impl Drawable
    where
        Self: Sized,
    {
        struct WithBg<'a, R: Drawable> {
            bg: Color,
            drawable: &'a R,
        }
        impl<R: Drawable> Drawable for WithBg<'_, R> {
            fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
                let mut adapter = Adapter {
                    placer,
                    bg: self.bg,
                };
                self.drawable.draw(&mut adapter, pos, blend)
            }

            fn size(&self, input: Size) -> Size {
                self.drawable.size(input)
            }
        }

        struct Adapter<'a> {
            placer: &'a mut dyn Placer,
            bg: Color,
        }

        impl Placer for Adapter<'_> {
            fn put(&mut self, pos: Position, pixel: Pixel, blend: BlendMode) {
                self.placer.put(pos, pixel.bg(self.bg), blend);
            }
            fn size(&self) -> Size {
                self.placer.size()
            }
        }

        WithBg {
            bg: bg.into(),
            drawable: self,
        }
    }

    fn with_offset(&self, offset: Position) -> impl Drawable
    where
        Self: Sized,
    {
        struct WithOffset<'a, R: Drawable> {
            offset: Position,
            drawable: &'a R,
        }

        impl<R: Drawable> Drawable for WithOffset<'_, R> {
            fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
                self.drawable.draw(placer, pos + self.offset, blend);
            }

            fn size(&self, input: Size) -> Size {
                self.drawable.size(input)
            }
        }

        WithOffset {
            offset,
            drawable: self,
        }
    }

    fn with_anchor(&self, anchor: Anchor2) -> impl Drawable
    where
        Self: Sized,
    {
        struct WithAnchor<'a, R: Drawable> {
            anchor: Anchor2,
            drawable: &'a R,
        }

        impl<R: Drawable> Drawable for WithAnchor<'_, R> {
            fn draw(&self, placer: &mut dyn Placer, pos: Position, blend: BlendMode) {
                let offset = self.size(placer.size()).to_position().to_signed();
                let pos = pos + offset;

                self.drawable.draw(placer, pos, blend);
            }

            fn size(&self, input: Size) -> Size {
                let child = self.drawable.size(input);
                let new = input * self.anchor - child * self.anchor;
                new.to_signed().to_unsigned()
            }
        }

        WithAnchor {
            anchor,
            drawable: self,
        }
    }
}

impl<T> DrawableExt for T where T: Drawable {}
