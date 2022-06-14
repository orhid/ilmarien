pub trait Unit<T> {
    fn confine(value: T) -> Self;
    fn release(self) -> T;
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Elevation(f64);

impl Elevation {
    pub fn meters(self) -> i32 {
        (self.0 * 13824.) as i32
    }
}

impl Unit<f64> for Elevation {
    fn confine(value: f64) -> Self {
        Self(value)
    }

    fn release(self) -> f64 {
        self.0
    }
}
