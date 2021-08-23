use geo::Coordinate;
use std::{
    convert::From,
    ops::{Add, Div, Mul, Neg, Rem, Sub},
};

macro_rules! impl_ops_internal {
    ($dat:ty, $trait: ident, $op: tt, $method: ident) => {
        impl $trait for $dat {
            type Output = Self;

            fn $method(self, other: Self) -> Self::Output {
               Self {x: self.x $op other.x, y: self.y $op other.y}
            }
        }
    };
}

macro_rules! impl_ops_external {
    ($dat:ty, $num:ty, $trait: ident, $op: tt, $method: ident) => {
        impl $trait<$num> for $dat {
            type Output = Self;

            fn $method(self, other: $num) -> Self::Output {
               Self {x: self.x $op other, y: self.y $op other}
            }
        }
    };
}

macro_rules! impl_dat {
    ($dat:ty, $num:ty) => {
        impl $dat {
            fn new(x: $num, y: $num) -> Self {
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

        impl_ops_internal!($dat, Add, +, add);
        impl_ops_internal!($dat, Sub, -, sub);
        impl_ops_external!($dat, $num, Mul, *, mul);
        impl_ops_external!($dat, $num, Div, /, div);
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DatumZa {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DatumRe {
    pub x: f64,
    pub y: f64,
}

impl_dat!(DatumZa, i32);
impl_dat!(DatumRe, f64);

impl Rem<i32> for DatumZa {
    type Output = Self;

    fn rem(self, other: i32) -> Self::Output {
        Self {
            x: self.x.rem_euclid(other),
            y: self.y.rem_euclid(other),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /*
    test carto::datum::test::dat_re_from ... ok
    test carto::datum::test::dat_re_new ... ok
    test carto::datum::test::dat_re_op_div ... ok
    test carto::datum::test::dat_re_op_add ... ok
    test carto::datum::test::dat_re_op_sub ... ok
    test carto::datum::test::dat_re_op_mul ... ok
    test carto::datum::test::dat_za_from ... ok
    test carto::datum::test::dat_za_new ... ok
    test carto::datum::test::dat_za_op_add ... ok
    test carto::datum::test::dat_za_op_div ... ok
    test carto::datum::test::dat_za_op_mul ... ok
    test carto::datum::test::dat_za_op_rem ... ok
    test carto::datum::test::dat_za_op_sub ... ok
    test carto::datum::test::datum_cast ... ok
    test carto::datum::test::datum_cast_find ... ok
    test carto::brane::test::brane_type_conversion ... ok
    test carto::datum::test::datum_find ... ok
    test carto::datum::test::datum_find_cast ... ok
    test carto::datum::test::datum_re2za ... ok
    test carto::datum::test::res_deref ... ok
    test carto::datum::test::res_ops_div ... ok
    test carto::datum::test::res_ops_mul ... ok
    test carto::datum::test::res_ops_rem ... ok
    */

    macro_rules! test_new {
        ($name: ident, $dat:ty, $x:expr, $y:expr) => {
            #[test]
            fn $name() {
                let dat = <$dat>::new($x, $y);
                assert_eq!(dat.x, $x);
                assert_eq!(dat.y, $y);
            }
        };
    }

    macro_rules! test_from {
        ($name: ident, $dat:ident, $num:ty, $x:expr, $y:expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    Coordinate { x: $x, y: $y },
                    Coordinate::<$num>::from($dat { x: $x, y: $y })
                );
                assert_eq!(Coordinate { x: $x, y: $y }, $dat { x: $x, y: $y }.into());
            }
        };
    }

    macro_rules! test_ops_internal {
        ($name: ident, $dat: ident, $op: tt, $sx: expr, $sy :expr, $ox: expr, $oy: expr, $rx: expr, $ry: expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    $dat{x: $sx, y: $sy} $op $dat{x: $ox, y: $oy},
                    $dat{x: $rx ,y: $ry},
                );
            }
        };
    }

    macro_rules! test_ops_external {
        ($name: ident, $dat: ident, $op: tt, $sx: expr, $sy :expr, $o: expr, $rx: expr, $ry: expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    $dat{x: $sx, y: $sy} $op $o,
                    $dat{x: $rx ,y: $ry},
                );
            }
        };
    }

    test_new!(dcs_new, DatumZa, 0, 1);
    test_new!(cnt_new, DatumRe, 0.0, 1.0);
    test_from!(dsc_from, DatumZa, i32, 0, 1);
    test_from!(cnt_from, DatumRe, f64, 0.0, 1.0);
    test_ops_internal!(dsc_op_add, DatumZa, +, 0, 1, 2, 3, 2, 4);
    test_ops_internal!(dsc_op_sub, DatumZa, -, 0, 1, 2, 3, -2, -2);
    test_ops_internal!(cnt_op_add, DatumRe, +, 0.0, 1.0, 2.0, 3.0, 2.0, 4.0);
    test_ops_internal!(cnt_op_sub, DatumRe, -, 0.0, 1.0, 2.0, 3.0, -2.0, -2.0);
    test_ops_external!(dsc_op_mul, DatumZa, *, 1, 2, 3, 3, 6);
    test_ops_external!(dsc_op_div, DatumZa, /, 6, 3, 3, 2, 1);
    test_ops_external!(cnt_op_mul, DatumRe, *, 1.0, 2.0, 3.0, 3.0, 6.0);
    test_ops_external!(cnt_op_div, DatumRe, /, 6.0, 3.0, 3.0, 2.0, 1.0);

    #[test]
    fn dsc_op_rem() {
        assert_eq!(DatumZa { x: 5, y: -1 } % 4, DatumZa { x: 1, y: 3 });
    }
}
