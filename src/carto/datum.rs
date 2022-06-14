use geo::Coordinate;
use ord_subset::OrdSubsetIterExt;
use std::{
    convert::From,
    ops::{Add, Div, Mul, Neg, Rem, Sub},
};

macro_rules! impl_op_internal {
    ($dat:ty, $trait: ident, $op: tt, $method: ident) => {
        impl $trait for $dat {
            type Output = Self;

            fn $method(self, other: Self) -> Self::Output {
               Self {x: self.x $op other.x, y: self.y $op other.y}
            }
        }
    };
}

macro_rules! impl_op_external {
    ($dat:ty, $num:ty, $trait: ident, $op: tt, $method: ident) => {
        impl $trait<$num> for $dat {
            type Output = Self;

            fn $method(self, other: $num) -> Self::Output {
               Self {x: self.x $op other, y: self.y $op other}
            }
        }
    };
}

macro_rules! impl_datum {
    ($dat:ty, $num:ty) => {
        impl $dat {
            pub fn new(x: $num, y: $num) -> Self {
                Self { x, y }
            }
        }

        impl From<$dat> for Coordinate<$num> {
            fn from(dat: $dat) -> Self {
                Self { x: dat.x, y: dat.y }
            }
        }

        impl Neg for $dat {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    x: -self.x,
                    y: -self.y,
                }
            }
        }

        impl_op_internal!($dat, Add, +, add);
        impl_op_internal!($dat, Sub, -, sub);
        impl_op_external!($dat, $num, Mul, *, mul);
        impl_op_external!($dat, $num, Div, /, div);
    };
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DatumZa {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DatumRe {
    pub x: f64,
    pub y: f64,
}

impl_datum!(DatumZa, i32);
impl_datum!(DatumRe, f64);

impl Rem<i32> for DatumZa {
    type Output = Self;

    fn rem(self, other: i32) -> Self::Output {
        Self {
            x: self.x.rem_euclid(other),
            y: self.y.rem_euclid(other),
        }
    }
}

impl From<DatumZa> for DatumRe {
    fn from(datum: DatumZa) -> Self {
        Self {
            x: datum.x as f64,
            y: datum.y as f64,
        }
    }
}

impl From<DatumRe> for DatumZa {
    fn from(datum: DatumRe) -> Self {
        let candidate = *datum
            .rhombus()
            .iter()
            .ord_subset_min_by_key(|g| (datum.x - g.x as f64).abs() + (datum.y - g.y as f64).abs())
            .unwrap();
        Self {
            x: candidate.x as i32,
            y: candidate.y as i32,
        }
    }
}

impl DatumZa {
    /// transform into a Real Datum inside the unit square
    pub fn cast(self, resolution: usize) -> DatumRe {
        DatumRe::from(self) / resolution as f64
    }

    /// create from a linear index
    pub fn enravel(index: usize, resolution: usize) -> Self {
        Self {
            x: (index / resolution) as i32,
            y: (index % resolution) as i32,
        }
    }

    /// transform into a linear index
    pub fn unravel(self, resolution: usize) -> usize {
        self.x as usize * resolution + self.y as usize
    }

    /// transform into a linear index carefully
    pub fn unravel_safe(self, resolution: usize) -> usize {
        self.x as usize % resolution * resolution + self.y as usize % resolution
    }
}

impl DatumRe {
    /// transform into a Zahl Datum
    pub fn find(self, resolution: usize) -> DatumZa {
        DatumZa::from(self * resolution as f64)
    }

    /// transform into a Zahl Datum faster by simply flooring
    pub fn floor(self, resolution: usize) -> DatumZa {
        DatumZa {
            x: (self.x * resolution as f64) as i32,
            y: (self.y * resolution as f64) as i32,
        }
    }

    /// four surrounding Zahl Data
    pub fn rhombus(self) -> [DatumZa; 4] {
        let xfl = self.x as i32;
        let yfl = self.y as i32;
        [
            DatumZa { x: xfl, y: yfl },
            DatumZa { x: xfl + 1, y: yfl },
            DatumZa { x: xfl, y: yfl + 1 },
            DatumZa {
                x: xfl + 1,
                y: yfl + 1,
            },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /* # data */

    macro_rules! test_datum_new {
        ($name: ident, $datum:ty, $x:expr, $y:expr) => {
            #[test]
            fn $name() {
                let datum = <$datum>::new($x, $y);
                assert_eq!(datum.x, $x);
                assert_eq!(datum.y, $y);
            }
        };
    }

    macro_rules! test_datum_from {
        ($name: ident, $datum:ident, $num:ty, $x:expr, $y:expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    Coordinate { x: $x, y: $y },
                    Coordinate::<$num>::from($datum { x: $x, y: $y })
                );
                assert_eq!(Coordinate { x: $x, y: $y }, $datum { x: $x, y: $y }.into());
            }
        };
    }

    macro_rules! test_datum_op_internal {
        ($name: ident, $datum: ident, $op: tt, $sx: expr, $sy :expr, $ox: expr, $oy: expr, $rx: expr, $ry: expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    $datum{x: $sx, y: $sy} $op $datum{x: $ox, y: $oy},
                    $datum{x: $rx ,y: $ry},
                );
            }
        };
    }

    macro_rules! test_datum_op_external {
        ($name: ident, $datum: ident, $op: tt, $sx: expr, $sy :expr, $o: expr, $rx: expr, $ry: expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    $datum{x: $sx, y: $sy} $op $o,
                    $datum{x: $rx ,y: $ry},
                );
            }
        };
    }

    test_datum_new!(datum_za_new, DatumZa, 0, 1);
    test_datum_new!(datum_re_new, DatumRe, 0.0, 1.0);
    test_datum_from!(datum_za_into_coordinate, DatumZa, i32, 0, 1);
    test_datum_from!(datum_re_into_coordinate, DatumRe, f64, 0.0, 1.0);
    test_datum_op_internal!(datum_za_op_add, DatumZa, +, 0, 1, 2, 3, 2, 4);
    test_datum_op_internal!(datum_za_op_sub, DatumZa, -, 0, 1, 2, 3, -2, -2);
    test_datum_op_internal!(datum_re_op_add, DatumRe, +, 0.0, 1.0, 2.0, 3.0, 2.0, 4.0);
    test_datum_op_internal!(datum_re_op_sub, DatumRe, -, 0.0, 1.0, 2.0, 3.0, -2.0, -2.0);
    test_datum_op_external!(datum_za_op_mul, DatumZa, *, 1, 2, 3, 3, 6);
    test_datum_op_external!(datum_za_op_div, DatumZa, /, 6, 3, 3, 2, 1);
    test_datum_op_external!(datum_re_op_mul, DatumRe, *, 1.0, 2.0, 3.0, 3.0, 6.0);
    test_datum_op_external!(datum_re_op_div, DatumRe, /, 6.0, 3.0, 3.0, 2.0, 1.0);

    #[test]
    fn datum_za_op_rem() {
        assert_eq!(DatumZa { x: 5, y: -1 } % 4, DatumZa { x: 1, y: 3 });
    }

    #[test]
    fn datum_re2za() {
        assert_eq!(
            DatumZa::from(DatumRe::new(0.1, 0.1)),
            DatumZa { x: 0, y: 0 },
        );
        assert_eq!(
            DatumZa::from(DatumRe::new(0.1, 0.9)),
            DatumZa { x: 0, y: 1 },
        );
        assert_eq!(
            DatumZa::from(DatumRe::new(0.9, 0.1)),
            DatumZa { x: 1, y: 0 },
        );
        assert_eq!(
            DatumZa::from(DatumRe::new(0.9, 0.9)),
            DatumZa { x: 1, y: 1 },
        );
    }

    #[test]
    fn datum_cast() {
        assert_eq!(DatumZa::new(0, 0).cast(4), DatumRe::new(0.0, 0.0));
        assert_eq!(DatumZa::new(0, 1).cast(4), DatumRe::new(0.0, 0.25));
        assert_eq!(DatumZa::new(1, 0).cast(4), DatumRe::new(0.25, 0.0));
    }

    #[test]
    fn datum_find() {
        assert_eq!(DatumRe::new(0.0, 0.0).find(4), DatumZa::new(0, 0));
        assert_eq!(DatumRe::new(0.0, 0.25).find(4), DatumZa::new(0, 1));
        assert_eq!(DatumRe::new(0.25, 0.0).find(4), DatumZa::new(1, 0));
    }

    #[test]
    fn datum_cast_find() {
        let datum = DatumZa::new(0, 1);
        assert_eq!(datum.cast(4).find(4), datum);
        let datum = DatumZa::new(1, 0);
        assert_eq!(datum.cast(4).find(4), datum);
    }

    #[test]
    fn datum_find_cast() {
        let datum = DatumRe::new(0.0, 0.25);
        assert_eq!(datum.find(4).cast(4), datum);
        let datum = DatumRe::new(0.25, 0.0);
        assert_eq!(datum.find(4).cast(4), datum);
    }

    #[test]
    fn datum_enravel() {
        assert_eq!(DatumZa::enravel(0, 4), DatumZa::new(0, 0));
        assert_eq!(DatumZa::enravel(1, 4), DatumZa::new(0, 1));
        assert_eq!(DatumZa::enravel(4, 4), DatumZa::new(1, 0));
    }

    #[test]
    fn datum_unravel() {
        assert_eq!(DatumZa::new(0, 0).unravel(4), 0);
        assert_eq!(DatumZa::new(0, 1).unravel(4), 1);
        assert_eq!(DatumZa::new(1, 0).unravel(4), 4);
    }

    #[test]
    fn datum_enravel_unravel() {
        assert_eq!(DatumZa::enravel(0, 4).unravel(4), 0);
        assert_eq!(DatumZa::enravel(1, 4).unravel(4), 1);
        assert_eq!(DatumZa::enravel(4, 4).unravel(4), 4);
    }

    #[test]
    fn datum_unravel_enravel() {
        let datum = DatumZa::new(0, 1);
        assert_eq!(DatumZa::enravel(datum.unravel(4), 4), datum);
        let datum = DatumZa::new(1, 0);
        assert_eq!(DatumZa::enravel(datum.unravel(4), 4), datum);
    }
}
