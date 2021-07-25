use geo::Coordinate;
use std::f64::consts::TAU;
const SQRT3: f64 = 1.7320508;

/* local */

pub trait Gon {
    fn neighbour(&self, n: usize, modulo: i32) -> Self;

    fn ambit(&self, modulo: i32) -> Vec<Self>
    where
        Self: Sized,
    {
        (0..6).map(|n| self.neighbour(n, modulo)).collect()
    }

    fn centre(&self) -> Coordinate<f64>;

    fn corner(&self, centre: Coordinate<f64>, n: usize) -> Coordinate<f64>;

    fn corners(&self) -> Vec<Coordinate<f64>> {
        let centre = self.centre();
        (0..6).map(|n| self.corner(centre, n)).collect()
    }
}

impl Gon for Coordinate<i32> {
    fn neighbour(&self, n: usize, modulo: i32) -> Self {
        match n % 6 {
            0 => Coordinate {
                x: (self.x + 1).rem_euclid(modulo),
                y: self.y,
            },
            1 => Coordinate {
                x: (self.x + 1).rem_euclid(modulo),
                y: (self.y - 1).rem_euclid(modulo),
            },
            2 => Coordinate {
                x: self.x,
                y: (self.y - 1).rem_euclid(modulo),
            },
            3 => Coordinate {
                x: (self.x - 1).rem_euclid(modulo),
                y: self.y,
            },
            4 => Coordinate {
                x: (self.x - 1).rem_euclid(modulo),
                y: (self.y + 1).rem_euclid(modulo),
            },
            5 => Coordinate {
                x: self.x,
                y: (self.y + 1).rem_euclid(modulo),
            },
            _ => Coordinate {
                x: self.x,
                y: self.y,
            },
        }
    }

    fn centre(&self) -> Coordinate<f64> {
        Coordinate {
            x: self.x as f64 * 1.5,
            y: self.x as f64 * SQRT3 / 2.0 + self.y as f64 * SQRT3,
        }
    }

    fn corner(&self, centre: Coordinate<f64>, n: usize) -> Coordinate<f64> {
        let angle = (n % 6) as f64 * TAU / 6.0;
        centre
            + Coordinate {
                x: angle.cos(),
                y: angle.sin(),
            }
    }
}

/* global */

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

impl Tileable for Coordinate<i32> {
    fn tile(&self, modulo: i32) -> Tile {
        if 2 * self.x + self.y - modulo <= 0 {
            if self.x + 2 * self.y - modulo < 0 {
                Tile::Y
            } else {
                Tile::R
            }
        } else {
            if 2 * self.x + self.y - 2 * modulo <= 0 {
                if self.x - self.y <= 0 {
                    Tile::R
                } else {
                    Tile::B
                }
            } else {
                if self.x + 2 * self.y - 2 * modulo < 0 {
                    Tile::B
                } else {
                    Tile::G
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /* local */

    #[test]
    fn neighbour() {
        let org = Coordinate { x: 0, y: 0 };
        assert_eq!(org.neighbour(0, 4), Coordinate { x: 1, y: 0 });
        assert_eq!(org.neighbour(2, 4), Coordinate { x: 0, y: 3 });
        assert_eq!(org.neighbour(3, 4), Coordinate { x: 3, y: 0 });
        assert_eq!(org.neighbour(5, 4), Coordinate { x: 0, y: 1 });
    }

    #[test]
    fn ambit() {
        let org = Coordinate { x: 0, y: 0 };
        let amb = org.ambit(4);
        assert_eq!(amb.len(), 6);
        for j in 0..6 {
            assert_eq!(amb[j], org.neighbour(j, 4));
        }
    }

    #[test]
    fn centre() {
        assert_eq!(
            Coordinate { x: 1, y: 0 }.centre(),
            Coordinate {
                x: 1.5,
                y: SQRT3 / 2.0
            }
        );
        assert_eq!(
            Coordinate { x: 0, y: 1 }.centre(),
            Coordinate { x: 0.0, y: SQRT3 }
        );
    }

    #[test]
    fn corners() {
        assert_eq!(
            Coordinate { x: 0, y: 0 }.corners()[0],
            Coordinate { x: 1.0, y: 0.0 }
        );
    }

    /* global */

    #[test]
    fn tile_placement() {
        assert_eq!(Coordinate { x: 0, y: 0 }.tile(4), Tile::Y);
        assert_eq!(Coordinate { x: 1, y: 1 }.tile(4), Tile::Y);
        assert_eq!(Coordinate { x: 3, y: 1 }.tile(4), Tile::B);
        assert_eq!(Coordinate { x: 1, y: 3 }.tile(4), Tile::R);
        assert_eq!(Coordinate { x: 3, y: 3 }.tile(4), Tile::G);
        assert_eq!(Coordinate { x: 4, y: 4 }.tile(4), Tile::G);
    }
}
