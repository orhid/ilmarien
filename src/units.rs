use std::ops::{Add, Div, Mul, Neg, Sub};

pub trait Unit {
    type Raw;

    fn confine(value: Self::Raw) -> Self;
    fn release(self) -> Self::Raw;
}

macro_rules! impl_op_internal {
    ($unit:ident, $trait: ident, $method: ident) => {
        impl $trait for $unit {
            type Output = Self;

            fn $method(self, other: Self) -> Self::Output {
                Self(self.0.$method(other.0))
            }
        }
    };
}

macro_rules! impl_op_external {
    ($unit:ident, $raw: ty, $trait: ident, $method: ident) => {
        impl $trait<$raw> for $unit {
            type Output = Self;

            fn $method(self, other: $raw) -> Self::Output {
                Self(self.0.$method(other))
            }
        }
    };
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

        impl Neg for $unit {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }

        impl_op_internal!($unit, Add, add);
        impl_op_internal!($unit, Sub, sub);
        impl_op_external!($unit, $raw, Mul, mul);
        impl_op_external!($unit, $raw, Div, div);
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
        -27.
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
    pub fn milimeters(self) -> u16 {
        (self.0 * 324.).max(0.) as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_unit_op_internal {
        ($name: ident, $unit: ident, $op: tt, $s: expr, $o: expr, $r: expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    $unit::confine($s) $op $unit::confine($o),
                    $unit::confine($r),
                );
            }
        };
    }

    macro_rules! test_unit_op_external {
        ($name: ident, $unit: ident, $op: tt, $s: expr, $o: expr, $r: expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    $unit::confine($s) $op $o,
                    $unit::confine($r),
                );
            }
        };
    }

    test_unit_op_internal!(elevation_op_add, Elevation, +, 0., 1., 1.);
    test_unit_op_internal!(elevation_op_sub, Elevation, -, 2., 1., 1.);
    test_unit_op_external!(elevation_op_mul, Elevation, *, 2., 3., 6.);
    test_unit_op_external!(elevation_op_div, Elevation, /, 6., 3., 2.);
    test_unit_op_internal!(temperature_op_add, Temperature, +, 0., 1., 1.);
    test_unit_op_internal!(temperature_op_sub, Temperature, -, 2., 1., 1.);
    test_unit_op_external!(temperature_op_mul, Temperature, *, 2., 3., 6.);
    test_unit_op_external!(temperature_op_div, Temperature, /, 6., 3., 2.);
    test_unit_op_internal!(precipitation_op_add, Precipitation, +, 0., 1., 1.);
    test_unit_op_internal!(precipitation_op_sub, Precipitation, -, 2., 1., 1.);
    test_unit_op_external!(precipitation_op_mul, Precipitation, *, 2., 3., 6.);
    test_unit_op_external!(precipitation_op_div, Precipitation, /, 6., 3., 2.);
}
