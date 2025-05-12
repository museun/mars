use mars_math::{Position, Size};

use crate::{BlendMode, Color, Pixel, Rasterizer, Renderer, RendererSetup, ResizeMode, Surface};

use super::Placer;

#[derive(Debug)]
pub struct BasicRenderer {
    surface: Surface<Pixel>,
    size: Size,
    default_fg: Color,
    default_bg: Color,
}

impl BasicRenderer {
    pub fn new(size: Size) -> Self {
        Self {
            surface: Surface::new(size, Pixel::empty()),
            size,
            default_fg: Color::default(),
            default_bg: Color::default(),
        }
    }

    pub fn default_fg(mut self, default_fg: impl Into<Color>) -> Self {
        self.set_default_fg(default_fg);
        self
    }

    pub fn default_bg(mut self, default_bg: impl Into<Color>) -> Self {
        self.set_default_bg(default_bg);
        self
    }

    pub fn set_default_fg(&mut self, default_fg: impl Into<Color>) {
        self.default_fg = default_fg.into();
    }

    pub fn set_default_bg(&mut self, default_bg: impl Into<Color>) {
        self.default_bg = default_bg.into();
    }

    pub fn resize(&mut self, size: Size, _mode: ResizeMode) {
        if std::mem::replace(&mut self.size, size) == size {
            return;
        }
        self.surface.resize(size, ResizeMode::Discard);
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn put(&mut self, pos: Position, pixel: Pixel, blend: BlendMode) {
        let Some(pos) = pos.to_unsigned_checked() else {
            return;
        };

        if pos.x > self.size.width || pos.y >= self.size.height {
            return;
        }

        let old = &mut self.surface[pos];

        let left = old.background;
        let right = pixel.foreground;

        *old = pixel;
        // *old = Pixel {
        //     background: Pixel::blend_bg(left, right, self.default_bg, blend),
        //     ..pixel
        // }
    }

    pub fn render<R>(&mut self, mut rasterizer: R) -> Result<(), R::Error>
    where
        R: Rasterizer,
    {
        let mut data = [0u8; 4];
        let replacement = Pixel::empty() //
            .fg(self.default_fg)
            .bg(self.default_bg);
        let mut state = CursorState::default();

        rasterizer.begin()?;

        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let pos = Position::new(x, y);

                if state.maybe_move(pos) {
                    rasterizer.move_to(pos.to_signed())?;
                }

                let pixel = std::mem::replace(&mut self.surface[pos], replacement.clone());
                if let Some(fg) = state.maybe_fg(pixel.foreground.get_or_default(self.default_fg)) {
                    rasterizer.set_fg(fg)?;
                }
                if let Some(bg) = state.maybe_bg(pixel.background.get_or_default(self.default_bg)) {
                    rasterizer.set_bg(bg)?;
                }

                // TODO attributes

                match pixel.data {
                    crate::pixel::PixelData::Char(ch) => {
                        let s = ch.encode_utf8(&mut data);
                        rasterizer.write(s)?;
                    }
                    crate::pixel::PixelData::Str(s) => {
                        rasterizer.write(&s)?;
                    }
                };
            }
        }

        rasterizer.move_to(Position::ZERO)?;
        rasterizer.end()
    }
}

#[derive(Default)]
struct CursorState {
    previous: Option<Position<u32>>,
    prev_fg: Color,
    prev_bg: Color,
    // TODO attributes
}

impl CursorState {
    fn maybe_move(&mut self, pos: Position<u32>) -> bool {
        let should_move = match self.previous {
            Some(last) if last.y != pos.y || last.x != pos.x - 1 => true,
            None => true,
            _ => false,
        };
        self.previous = Some(pos);
        should_move
    }

    fn maybe_fg(&mut self, fg: Color) -> Option<Color> {
        if self.prev_fg != fg {
            self.prev_fg = fg;
            return Some(self.prev_fg);
        }
        None
    }

    fn maybe_bg(&mut self, bg: Color) -> Option<Color> {
        if self.prev_bg != bg {
            self.prev_bg = bg;
            return Some(self.prev_bg);
        }
        None
    }
}

impl Placer for BasicRenderer {
    fn put(&mut self, pos: Position, pixel: Pixel, blend: BlendMode) {
        Self::put(self, pos, pixel, blend);
    }

    fn size(&self) -> Size {
        Self::size(self)
    }
}

impl Renderer for BasicRenderer {
    fn get(&self, pos: Position) -> Option<&Pixel> {
        self.surface.get(pos)
    }

    fn get_mut(&mut self, pos: Position) -> Option<&mut Pixel> {
        self.surface.get_mut(pos)
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

impl RendererSetup for BasicRenderer {
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
