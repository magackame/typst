use super::*;

/// A size in 2D.
#[derive(Default, Copy, Clone, PartialEq)]
pub struct Size {
    /// The width.
    pub width: Length,
    /// The height.
    pub height: Length,
}

impl Size {
    /// The zero size.
    pub const ZERO: Self = Self {
        width: Length::ZERO,
        height: Length::ZERO,
    };

    /// Create a new size from width and height.
    pub fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }

    /// Whether the other size fits into this one (smaller width and height).
    pub fn fits(self, other: Self) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

impl Get<SpecAxis> for Size {
    type Component = Length;

    fn get(self, axis: SpecAxis) -> Length {
        match axis {
            SpecAxis::Horizontal => self.width,
            SpecAxis::Vertical => self.height,
        }
    }

    fn get_mut(&mut self, axis: SpecAxis) -> &mut Length {
        match axis {
            SpecAxis::Horizontal => &mut self.width,
            SpecAxis::Vertical => &mut self.height,
        }
    }
}

impl Switch for Size {
    type Other = Gen<Length>;

    fn switch(self, flow: Flow) -> Self::Other {
        match flow.main.axis() {
            SpecAxis::Horizontal => Gen::new(self.width, self.height),
            SpecAxis::Vertical => Gen::new(self.height, self.width),
        }
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({} x {})", self.width, self.height)
    }
}

impl Neg for Size {
    type Output = Self;

    fn neg(self) -> Self {
        Self { width: -self.width, height: -self.height }
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

sub_impl!(Size - Size -> Size);

impl Mul<f64> for Size {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            width: self.width * other,
            height: self.height * other,
        }
    }
}

impl Mul<Size> for f64 {
    type Output = Size;

    fn mul(self, other: Size) -> Size {
        other * self
    }
}

impl Div<f64> for Size {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Self {
            width: self.width / other,
            height: self.height / other,
        }
    }
}

assign_impl!(Size -= Size);
assign_impl!(Size += Size);
assign_impl!(Size *= f64);
assign_impl!(Size /= f64);