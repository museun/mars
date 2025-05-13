use mars_math::{Position, Size};

#[derive(Debug, Clone)]
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

    #[track_caller]
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
    #[track_caller]
    pub fn get(&self, pos: Position) -> Option<&T> {
        // FIXME check over/underflow
        let index = Self::pos_of(self.size.width, pos + self.pos) as usize;
        self.pixels.get(index)
    }

    #[inline(always)]
    #[track_caller]
    pub fn get_mut(&mut self, pos: Position) -> Option<&mut T> {
        // FIXME check over/underflow
        let index = Self::pos_of(self.size.width, pos + self.pos) as usize;
        self.pixels.get_mut(index)
    }

    #[track_caller]
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
            (Position::new(x as _, y as _), p)
        })
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (Position, &mut T)> + DoubleEndedIterator {
        self.pixels.iter_mut().enumerate().map(|(i, p)| {
            let x = i as u32 % self.size.width;
            let y = i as u32 / self.size.width;
            (Position::new(x as _, y as _), p)
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

impl<T> std::ops::Index<Position<i32>> for Surface<T> {
    type Output = T;
    #[track_caller]
    #[inline]
    fn index(&self, index: Position<i32>) -> &Self::Output {
        assert!(index.x >= 0, "pos.x cannot be negative");
        assert!(index.y >= 0, "pos.y cannot be negative");
        let index = Self::pos_of(self.size.width, index + self.pos) as usize;
        &self.pixels[index]
    }
}

impl<T> std::ops::IndexMut<Position<i32>> for Surface<T> {
    #[track_caller]
    #[inline]
    fn index_mut(&mut self, index: Position<i32>) -> &mut Self::Output {
        assert!(index.x >= 0, "pos.x cannot be negative");
        assert!(index.y >= 0, "pos.y cannot be negative");
        let index = Self::pos_of(self.size.width, index + self.pos) as usize;
        &mut self.pixels[index]
    }
}

impl<T> std::ops::Index<Position<u32>> for Surface<T> {
    type Output = T;
    #[track_caller]
    #[inline]
    fn index(&self, index: Position<u32>) -> &Self::Output {
        let index = Position::new(index.x as _, index.y as _);
        let index = Self::pos_of(self.size.width, index + self.pos) as usize;
        &self.pixels[index]
    }
}

impl<T> std::ops::IndexMut<Position<u32>> for Surface<T> {
    #[track_caller]
    #[inline]
    fn index_mut(&mut self, index: Position<u32>) -> &mut Self::Output {
        let index = Position::new(index.x as _, index.y as _);
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
