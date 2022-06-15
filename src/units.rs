pub trait Unit {
    type Raw;

    fn confine(value: Self::Raw) -> Self;
    fn release(self) -> Self::Raw;
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Elevation(f64);

impl Elevation {
    pub fn meters(self) -> i32 {
        (self.0 * 13824.) as i32
    }
}

impl Unit for Elevation {
    type Raw = f64;
    fn confine(value: Self::Raw) -> Self {
        Self(value)
    }

    fn release(self) -> Self::Raw {
        self.0
    }
}
