#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Anchor {
    #[default]
    Min,
    Center,
    Max,
}

impl Anchor {
    pub const LEFT: Self = Self::Min;
    pub const RIGHT: Self = Self::Max;

    pub const CENTER: Self = Self::Center;

    pub const TOP: Self = Self::Min;
    pub const BOTTOM: Self = Self::Max;
}

impl Anchor {
    pub const fn factor(&self) -> f64 {
        match self {
            Self::Min => 0.0,
            Self::Center => 0.5,
            Self::Max => 1.0,
        }
    }

    pub const fn align(&self, available: f64, size: f64) -> f64 {
        (available - size) * self.factor()
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Anchor2 {
    pub x: Anchor,
    pub y: Anchor,
}

impl Anchor2 {
    pub const fn factor(&self) -> (f64, f64) {
        (self.x.factor(), self.y.factor())
    }
}

impl Anchor2 {
    pub const LEFT_TOP: Self = Self {
        x: Anchor::LEFT,
        y: Anchor::TOP,
    };

    pub const RIGHT_TOP: Self = Self {
        x: Anchor::RIGHT,
        y: Anchor::TOP,
    };

    pub const CENTER_TOP: Self = Self {
        x: Anchor::CENTER,
        y: Anchor::TOP,
    };

    pub const LEFT_CENTER: Self = Self {
        x: Anchor::LEFT,
        y: Anchor::CENTER,
    };

    pub const CENTER_CENTER: Self = Self {
        x: Anchor::CENTER,
        y: Anchor::CENTER,
    };

    pub const RIGHT_CENTER: Self = Self {
        x: Anchor::RIGHT,
        y: Anchor::CENTER,
    };

    pub const LEFT_BOTTOM: Self = Self {
        x: Anchor::LEFT,
        y: Anchor::BOTTOM,
    };

    pub const CENTER_BOTTOM: Self = Self {
        x: Anchor::CENTER,
        y: Anchor::BOTTOM,
    };

    pub const RIGHT_BOTTOM: Self = Self {
        x: Anchor::RIGHT,
        y: Anchor::BOTTOM,
    };
}
