pub trait Unit {
    type Raw;

    fn confine(value: Self::Raw) -> Self;
    fn release(self) -> Self::Raw;
}

macro_rules! impl_unit {
    ($unit:ident, $raw:ty) => {
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        pub struct $unit($raw);

        impl Unit for $unit {
            type Raw = $raw;

            fn confine(value: Self::Raw) -> Self {
                Self(value)
            }

            fn release(self) -> Self::Raw {
                self.0
            }
        }
    };
}

/* # elevation */

impl_unit!(Elevation, f64);

impl Elevation {
    pub fn meters(self) -> i32 {
        (self.0 * 13824.) as i32
    }
}

/* # temperature */

impl_unit!(Temperature, f64);

impl Temperature {
    pub const fn celcius_range() -> f64 {
        72.
    }

    pub const fn celcius_min() -> f64 {
        -36.
    }

    pub fn celcius_max() -> f64 {
        Self::celcius_range() + Self::celcius_min()
    }

    pub fn from_celcius(value: f64) -> Self {
        Self(value.mul_add(
            Self::celcius_range().recip(),
            -Self::celcius_min() * Self::celcius_range().recip(),
        ))
    }

    pub fn celcius(self) -> f64 {
        self.0.mul_add(Self::celcius_range(), Self::celcius_min())
    }
}

/* # precipitation */

impl_unit!(Precipitation, f64);

impl Precipitation {
    pub fn milimeters(self) -> i32 {
        (self.0 * 1.) as i32
    }
}
