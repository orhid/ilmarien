use crate::carto::datum::{DatumRe, DatumZa};
use std::f64::consts::TAU;

const SQRT3: f64 = 1.7320508;

/* # local */

/* ## direction */

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Direction {
    Xp,
    Zn,
    Yp,
    Xn,
    Zp,
    Yn,
}

impl From<usize> for Direction {
    fn from(direction: usize) -> Direction {
        match direction.rem_euclid(6) {
            0 => Direction::Xp,
            1 => Direction::Zn,
            2 => Direction::Yp,
            3 => Direction::Xn,
            4 => Direction::Zp,
            5 => Direction::Yn,
            _ => panic!("impossible result"),
        }
    }
}

impl From<Direction> for usize {
    fn from(direction: Direction) -> usize {
        match direction {
            Direction::Xp => 0,
            Direction::Zn => 1,
            Direction::Yp => 2,
            Direction::Xn => 3,
            Direction::Zp => 4,
            Direction::Yn => 5,
        }
    }
}

impl Direction {
    pub fn array() -> [Direction; 6] {
        [
            Direction::Xp,
            Direction::Zn,
            Direction::Yp,
            Direction::Xn,
            Direction::Zp,
            Direction::Yn,
        ]
    }
}

/* ## honeycombs */

/// honeycomb extending infinitely in all directions
pub trait HoneyCellPlanar {
    fn neighbour_planar(&self, direction: Direction) -> Self;

    fn ambit_planar(&self) -> [Self; 6]
    where
        Self: Sized,
    {
        Direction::array().map(|direction| self.neighbour_planar(direction))
    }

    fn ring_planar(&self, radius: i32) -> Vec<Self>
    where
        Self: Sized;

    fn ball_planar(&self, radius: i32) -> Vec<Self>
    where
        Self: Sized + Copy,
    {
        let mut ball = vec![*self];
        for j in 1..=radius {
            ball.append(&mut self.ring_planar(j));
        }
        ball
    }
}

impl HoneyCellPlanar for DatumZa {
    fn neighbour_planar(&self, direction: Direction) -> Self {
        match direction {
            Direction::Xp => DatumZa {
                x: (self.x + 1),
                y: self.y,
            },
            Direction::Zn => DatumZa {
                x: (self.x + 1),
                y: (self.y - 1),
            },
            Direction::Yp => DatumZa {
                x: self.x,
                y: (self.y - 1),
            },
            Direction::Xn => DatumZa {
                x: (self.x - 1),
                y: self.y,
            },
            Direction::Zp => DatumZa {
                x: (self.x - 1),
                y: (self.y + 1),
            },
            Direction::Yn => DatumZa {
                x: self.x,
                y: (self.y + 1),
            },
        }
    }

    fn ring_planar(&self, radius: i32) -> Vec<Self> {
        let mut gon = *self
            + DatumZa {
                x: (-radius),
                y: (radius),
            };
        let mut ring = Vec::<Self>::new();
        for direction in Direction::array() {
            for _ in 0..radius {
                ring.push(gon);
                gon = gon.neighbour_planar(direction);
            }
        }
        ring
    }
}

/// honeycomb wrapped around a torus
pub trait HoneyCellToroidal {
    fn neighbour_toroidal(&self, direction: Direction, modulo: i32) -> Self;

    fn ambit_toroidal(&self, modulo: i32) -> [Self; 6]
    where
        Self: Sized,
    {
        Direction::array().map(|direction| self.neighbour_toroidal(direction, modulo))
    }

    fn ring_toroidal(&self, radius: i32, modulo: i32) -> Vec<Self>
    where
        Self: Sized;

    fn ball_toroidal(&self, radius: i32, modulo: i32) -> Vec<Self>
    where
        Self: Sized + Copy,
    {
        let mut ball = vec![*self];
        for j in 1..=radius {
            ball.append(&mut self.ring_toroidal(j, modulo));
        }
        ball
    }

    fn dist_toroidal(&self, other: &Self, modulo: i32) -> i32;
}

impl HoneyCellToroidal for DatumZa {
    fn neighbour_toroidal(&self, direction: Direction, modulo: i32) -> Self {
        self.neighbour_planar(direction) % modulo
    }

    fn ring_toroidal(&self, radius: i32, modulo: i32) -> Vec<Self> {
        let mut gon = (*self
            + DatumZa {
                x: (-radius),
                y: (radius),
            })
            % modulo;
        let mut ring = Vec::<Self>::new();
        for direction in Direction::array() {
            for _ in 0..radius {
                ring.push(gon);
                gon = gon.neighbour_toroidal(direction, modulo);
            }
        }
        ring
    }

    fn dist_toroidal(&self, other: &Self, modulo: i32) -> i32 {
        [
            DatumZa { x: 0, y: 0 },
            DatumZa { x: modulo, y: 0 },
            DatumZa { x: 0, y: modulo },
            DatumZa {
                x: modulo,
                y: modulo,
            },
        ]
        .iter()
        .map(|z| {
            let d = *z - (*self - *other) % modulo;
            (d.x.abs() + d.y.abs() + (d.x + d.y).abs()) / 2
        })
        .min()
        .unwrap()
    }
}

pub fn ball_volume(radius: i32) -> i32 {
    3 * radius * (radius + 1) + 1
}

pub fn ball_cone_volume(radius: i32) -> i32 {
    (radius + 1).pow(3)
}

/* ## hexagons */

/// a representation of a honeycomb cell in a datum
pub trait Hexagon {
    fn centre(&self) -> DatumRe;

    fn corner(&self, centre: DatumRe, direction: Direction) -> DatumRe {
        let angle = usize::from(direction) as f64 * TAU * 6.0f64.recip();
        centre
            + DatumRe {
                x: angle.cos(),
                y: angle.sin(),
            }
    }

    fn corners(&self) -> [DatumRe; 6] {
        let centre = self.centre();
        Direction::array().map(|direction| self.corner(centre, direction))
    }
}

impl Hexagon for DatumZa {
    fn centre(&self) -> DatumRe {
        DatumRe {
            x: self.x as f64 * 1.5,
            y: self.x as f64 * SQRT3 * 2.0f64.recip() + self.y as f64 * SQRT3,
        }
    }
}

impl Hexagon for DatumRe {
    fn centre(&self) -> DatumRe {
        DatumRe {
            x: self.x * 1.5,
            y: self.x * SQRT3 * 2.0f64.recip() + self.y * SQRT3,
        }
    }
}

/* # global */

#[derive(Debug, PartialEq)]
pub enum Tile {
    R,
    G,
    B,
    Y,
}

pub trait Tileable {
    fn tile(&self, modulo: i32) -> Tile;
}

impl Tileable for DatumZa {
    fn tile(&self, modulo: i32) -> Tile {
        if 2 * self.x + self.y - modulo <= 0 {
            if self.x + 2 * self.y - modulo < 0 {
                Tile::Y
            } else {
                Tile::R
            }
        } else if 2 * self.x + self.y - 2 * modulo <= 0 {
            if self.x - self.y <= 0 {
                Tile::R
            } else {
                Tile::B
            }
        } else if self.x + 2 * self.y - 2 * modulo < 0 {
            Tile::B
        } else {
            Tile::G
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /* # local */

    /* ## planar */

    #[test]
    fn neighbour_planar() {
        let org = DatumZa { x: 0, y: 0 };
        assert_eq!(org.neighbour_planar(0.into()), DatumZa { x: 1, y: 0 });
        assert_eq!(org.neighbour_planar(2.into()), DatumZa { x: 0, y: -1 });
        assert_eq!(org.neighbour_planar(3.into()), DatumZa { x: -1, y: 0 });
        assert_eq!(org.neighbour_planar(5.into()), DatumZa { x: 0, y: 1 });
    }

    #[test]
    fn ambit_planar() {
        let org = DatumZa { x: 0, y: 0 };
        let ambit = org.ambit_planar();
        assert_eq!(ambit.len(), 6);
        for direction in Direction::array() {
            assert_eq!(
                ambit[usize::from(direction)],
                org.neighbour_planar(direction)
            );
        }
    }

    #[test]
    fn ring_planar() {
        let org = DatumZa { x: 0, y: 0 };
        let ambit = org.ambit_planar();
        let ring = org.ring_planar(1);
        for gon in &ambit {
            assert!(ring.contains(gon));
        }
        for gon in &ring {
            assert!(ambit.contains(gon));
        }
    }

    #[test]
    fn volume() {
        assert_eq!(ball_volume(0), 1);
        assert_eq!(ball_volume(1), 7);
        assert_eq!(ball_volume(2), 19);
        assert_eq!(ball_volume(3), 37);
    }

    #[test]
    fn volume_cone() {
        assert_eq!(ball_cone_volume(0), 1);
        assert_eq!(ball_cone_volume(1), 8);
        assert_eq!(ball_cone_volume(2), 27);
        assert_eq!(ball_cone_volume(3), 64);
    }

    /* ## toroidal */

    #[test]
    fn neighbour_toroidal() {
        let org = DatumZa { x: 0, y: 0 };
        assert_eq!(org.neighbour_toroidal(0.into(), 4), DatumZa { x: 1, y: 0 });
        assert_eq!(org.neighbour_toroidal(2.into(), 4), DatumZa { x: 0, y: 3 });
        assert_eq!(org.neighbour_toroidal(3.into(), 4), DatumZa { x: 3, y: 0 });
        assert_eq!(org.neighbour_toroidal(5.into(), 4), DatumZa { x: 0, y: 1 });
    }

    #[test]
    fn ambit_toroidal() {
        let org = DatumZa { x: 0, y: 0 };
        let amb = org.ambit_toroidal(4);
        assert_eq!(amb.len(), 6);
        for direction in Direction::array() {
            assert_eq!(
                amb[usize::from(direction)],
                org.neighbour_toroidal(direction, 4)
            );
        }
    }

    #[test]
    fn ring_toroidal() {
        let org = DatumZa { x: 0, y: 0 };
        let ambit = org.ambit_toroidal(4);
        let ring = org.ring_toroidal(1, 4);
        for gon in &ambit {
            assert!(ring.contains(gon));
        }
        for gon in &ring {
            assert!(ambit.contains(gon));
        }
    }

    #[test]
    fn dist_toroidal() {
        let z = DatumZa { x: 0, y: 0 };
        for n in z.ambit_toroidal(4) {
            assert_eq!(z.dist_toroidal(&n, 4), 1);
        }
        let z = DatumZa { x: 0, y: 3 };
        for n in z.ambit_toroidal(4) {
            assert_eq!(z.dist_toroidal(&n, 4), 1);
        }
    }

    /* ## hexagons */

    #[test]
    fn centre() {
        assert_eq!(
            DatumZa { x: 1, y: 0 }.centre(),
            DatumRe {
                x: 1.5,
                y: SQRT3 / 2.0
            }
        );
        assert_eq!(
            DatumZa { x: 0, y: 1 }.centre(),
            DatumRe { x: 0.0, y: SQRT3 }
        );
    }

    #[test]
    fn corners() {
        assert_eq!(
            DatumZa { x: 0, y: 0 }.corners()[0],
            DatumRe { x: 1.0, y: 0.0 }
        );
    }

    /* # global */

    #[test]
    fn tile_placement() {
        assert_eq!(DatumZa { x: 0, y: 0 }.tile(4), Tile::Y);
        assert_eq!(DatumZa { x: 1, y: 1 }.tile(4), Tile::Y);
        assert_eq!(DatumZa { x: 3, y: 1 }.tile(4), Tile::B);
        assert_eq!(DatumZa { x: 1, y: 3 }.tile(4), Tile::R);
        assert_eq!(DatumZa { x: 3, y: 3 }.tile(4), Tile::G);
        assert_eq!(DatumZa { x: 4, y: 4 }.tile(4), Tile::G);
    }
}
