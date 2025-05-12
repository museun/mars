use mars_math::{Position, Size};

use crate::{
    Attributes, BlendMode, Color, Pixel, Rasterizer, Renderer, RendererSetup, ResizeMode, Surface,
    pixel::PixelData,
};

use super::Placer;

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

    // TODO this is exclusive
    // there should be a different type for an inclusive index.

    // e.g. for y in 0..size.height { for x in 0..size.width { } }
    // vs
    // let size = surface.size();
    // surface.put(pos(size.width, size.height), ..) // bottom right
    pub const fn size(&self) -> Size {
        self.size
    }

    pub fn put(&mut self, pos: Position, pixel: Pixel) {
        let Ok(x) = u32::try_from(pos.x) else { return };
        let Ok(y) = u32::try_from(pos.y) else { return };
        if x > self.size.width || y >= self.size.height {
            return;
        }

        self.front[Position::new(x, y)].merge_mut(pixel);
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

    pub fn render<R>(&mut self, mut rasterizer: R) -> Result<(), R::Error>
    where
        R: Rasterizer,
    {
        let mut bytes = [0u8; 4];
        let mut state = State {
            fg: self.default_fg,
            bg: self.default_bg,
            ..State::default()
        };

        // BUG the diff totally isn't working
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let pos = Position::new(x as i32, y as i32); // FIXME this can lose resolution
                let pixel = std::mem::replace(&mut self.front[pos], Pixel::empty());

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
                        // let (reset, fg, bg) = state.reset_attr(fg, bg);
                        // if reset {
                        //     rasterizer.reset_attribute()?;
                        // }
                        // if let Some(bg) = bg {
                        //     rasterizer.set_bg(bg)?;
                        // }
                        // if let Some(fg) = fg {
                        //     rasterizer.set_fg(fg)?;
                        // }
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
            // std::mem::swap(&mut self.front, &mut self.back);
        }

        Ok(())
    }
}

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
        if self.attr == attr {
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

    // BUG this moves twice for 0,0
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

impl Placer for SurfaceRenderer {
    fn put(&mut self, pos: Position, pixel: Pixel, _blend: BlendMode) {
        Self::put(self, pos, pixel);
    }

    fn size(&self) -> Size {
        Self::size(self)
    }
}

impl Renderer for SurfaceRenderer {
    fn get(&self, pos: Position) -> Option<&Pixel> {
        self.front.get(pos)
    }

    fn get_mut(&mut self, pos: Position) -> Option<&mut Pixel> {
        self.front.get_mut(pos)
    }

    fn clear(&mut self) {
        self.fill(
            Position::ZERO,
            self.size(),
            Pixel::empty().fg(self.default_fg).bg(self.default_bg),
            BlendMode::Replace,
        );
    }

    fn render<R: Rasterizer>(&mut self, rasterizer: R) -> Result<(), R::Error> {
        Self::render(self, rasterizer)
    }
}

impl RendererSetup for SurfaceRenderer {
    fn default_colors(&self) -> (Color, Color) {
        (self.default_fg, self.default_bg)
    }

    fn set_default_fg(&mut self, default_fg: impl Into<Color>) {
        Self::set_default_fg(self, default_fg)
    }

    fn set_default_bg(&mut self, default_bg: impl Into<Color>) {
        Self::set_default_bg(self, default_bg)
    }
}
