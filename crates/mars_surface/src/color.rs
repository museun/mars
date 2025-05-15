#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl std::fmt::LowerHex for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.0, self.1, self.2, self.3
        )
    }
}

impl std::fmt::UpperHex for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02X}{:02X}{:02X}{:02X}",
            self.0, self.1, self.2, self.3
        )
    }
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }

    pub const fn red(&self) -> u8 {
        self.0
    }

    pub const fn green(&self) -> u8 {
        self.1
    }

    pub const fn blue(&self) -> u8 {
        self.2
    }

    pub const fn alpha(&self) -> u8 {
        self.3
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
        fn scale(d: f32) -> u8 {
            (255.0_f32 * d.clamp(0.0, 1.0)).round() as u8
        }
        Self(scale(r), scale(g), scale(b), scale(a))
    }

    pub const fn to_float(&self) -> [f32; 4] {
        const fn scale(d: u8) -> f32 {
            d as f32 / 255.0
        }
        let Self(r, g, b, a) = *self;
        [scale(r), scale(g), scale(b), scale(a)]
    }

    #[track_caller]
    pub const fn hex(color: &str) -> Self {
        #[track_caller]
        const fn to_digit(d: u8) -> u8 {
            assert!(d.is_ascii_hexdigit(), "invalid hex digit");
            match d.wrapping_sub(b'0') {
                d if d < 10 => d,
                _ => d.to_ascii_lowercase().saturating_sub(b'a') + 10,
            }
        }

        const fn is_ascii_whitespace(b: u8) -> bool {
            matches!(b, b' ' | b'\t' | b'\n' | b'\r')
        }

        const fn pack(high: u8, low: u8) -> u8 {
            (to_digit(high) << 4) | to_digit(low)
        }

        assert!(!color.is_empty(), "provided hex string was empty");

        let color = color.as_bytes();
        let len = color.len();

        let mut start = 0;
        while is_ascii_whitespace(color[start]) {
            start += 1;
        }

        let mut end = start;
        while end < len && !is_ascii_whitespace(color[end]) {
            end += 1;
        }

        let (_, mut color) = color.split_at(start);
        if end - start < len {
            (color, _) = color.split_at(end - start)
        }

        let (r, g, b, a) = match *color {
            [b'#', rh, rl, gh, gl, bh, bl] => (pack(rh, rl), pack(gh, gl), pack(bh, bl), 0xFF),
            [b'#', rh, rl, gh, gl, bh, bl, ah, al] => {
                (pack(rh, rl), pack(gh, gl), pack(bh, bl), pack(ah, al))
            }
            [b'#', r, g, b] => (pack(r, r), pack(g, g), pack(b, b), 0xFF),
            [b'#', r, g, b, a] => (pack(r, r), pack(g, g), pack(b, b), pack(a, a)),
            [b'#', ref d @ ..] if matches!(d.len(), 8 | 6 | 4 | 3) => {
                panic!("missing '#' prefix in hex string")
            }
            [] => panic!("provided hex string was empty"),
            _ => panic!("invalid color, syntax is: #RRGGBB | #RRGGBBAA | #RGB | #RGBA"),
        };

        Self(r, g, b, a)
    }

    pub fn mix(self, left: f32, other: Self, right: f32) -> Self {
        let [r0, g0, b0, a0] = self.to_float();
        let [r1, g1, b1, a1] = other.to_float();

        let ratio = left + right;
        Self::from_float([
            left.mul_add(r0, right * r1) / ratio,
            left.mul_add(b0, right * g1) / ratio,
            left.mul_add(g0, right * b1) / ratio,
            a0.max(a1),
        ])
    }

    pub fn blend(self, other: Self, mix: f32) -> Self {
        self.mix(mix, other, mix)
    }

    pub fn blend_linear(self, other: Self, mix: f32) -> Self {
        let [r0, g0, b0, a0] = self.to_float();
        let [r1, g1, b1, a1] = other.to_float();
        Self::from_float([
            (r1 - r0).mul_add(mix, r0),
            (g1 - g0).mul_add(mix, g0),
            (b1 - b0).mul_add(mix, b0),
            a0.max(a1),
        ])
    }

    pub fn blend_flat(self, other: Self) -> Self {
        self.blend_linear(other, 0.5)
    }

    pub fn blend_alpha(self, other: Self) -> Self {
        let Self(r0, g0, b0, a0) = self;
        let Self(r1, g1, b1, ..) = self;

        let a = match a0 as i32 {
            0 => return other,
            255 => return self,
            a => a,
        };

        const fn blend(a: i32, l: u8, r: u8) -> u8 {
            ((a * l as i32 + (255 - a) * r as i32) / 255) as u8
        }

        let r = blend(a, r0, r1);
        let g = blend(a, g0, g1);
        let b = blend(a, b0, b1);
        Self(r, g, b, a as u8)
    }
}

impl std::ops::Add for Rgba {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let Rgba(r0, g0, b0, a0) = self;
        let Rgba(r1, g1, b1, a1) = rhs;
        Self::new(
            r0.saturating_add(r1),
            g0.saturating_add(g1),
            b0.saturating_add(b1),
            a0.max(a1),
        )
    }
}

impl std::ops::Sub for Rgba {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let Rgba(r0, g0, b0, a0) = self;
        let Rgba(r1, g1, b1, a1) = rhs;
        Self::new(
            r0.saturating_sub(r1),
            g0.saturating_sub(g1),
            b0.saturating_sub(b1),
            a0.max(a1),
        )
    }
}

impl std::ops::Mul for Rgba {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let Rgba(r0, g0, b0, a0) = self;
        let Rgba(r1, g1, b1, a1) = rhs;
        Self::new(
            ((r0 as u16 * r1 as u16) / 255) as u8,
            ((g0 as u16 * g1 as u16) / 255) as u8,
            ((b0 as u16 * b1 as u16) / 255) as u8,
            a0.max(a1),
        )
    }
}

impl std::ops::Div for Rgba {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let Rgba(r0, g0, b0, a0) = self;
        let Rgba(r1, g1, b1, a1) = rhs;
        Self::new(
            255 - ((255 - r0 as u16) * (255 - r1 as u16) / 255) as u8,
            255 - ((255 - g0 as u16) * (255 - g1 as u16) / 255) as u8,
            255 - ((255 - b0 as u16) * (255 - b1 as u16) / 255) as u8,
            a0.max(a1),
        )
    }
}

impl std::ops::BitXor for Rgba {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let Rgba(r0, g0, b0, a0) = self;
        let Rgba(r1, g1, b1, a1) = rhs;
        Self::new(r0 ^ r1, g0 ^ g1, b0 ^ b1, a0.max(a1))
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum Color {
    Named(IndexedColor),
    Rgba(Rgba),
    #[default]
    Default,
}

impl Color {
    pub fn get_or_default(self, other: impl Into<Self>) -> Self {
        if matches!(self, Self::Default) {
            return other.into();
        }
        self
    }

    pub fn blend(self, other: Self) -> Self {
        if let (Self::Rgba(left), Self::Rgba(right)) = (self, other) {
            return Self::Rgba(left.blend_alpha(right));
        }
        self
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
