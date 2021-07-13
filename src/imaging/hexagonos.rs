use nalgebra::{Matrix2, Point2, Vector2};
use std::f32::consts::TAU;

pub trait Gon {
    fn neighbour(&self, n: usize, md: i32) -> Self;

    fn ambit(&self, md: i32) -> Vec<Self>
    where
        Self: Sized,
    {
        (0..6).map(|n| self.neighbour(n, md)).collect()
    }

    fn centre(&self) -> Point2<f32>;

    fn corner(&self, n: usize) -> Point2<f32>;

    fn corners(&self) -> Vec<Point2<f32>> {
        (0..6).map(|n| self.corner(n)).collect()
    }
}

const SQRT3: f32 = 1.7320508;
static CENTRE: Matrix2<f32> = Matrix2::new(1.5, 0.0, SQRT3 / 2.0, SQRT3);

fn int_to_float(v: &Vector2<i32>) -> Vector2<f32> {
    Vector2::new(v.x as f32, v.y as f32)
}

impl Gon for Vector2<i32> {
    fn neighbour(&self, n: usize, md: i32) -> Self {
        match n % 6 {
            0 => Vector2::new((self.x + 1).rem_euclid(md), self.y),
            1 => Vector2::new((self.x + 1).rem_euclid(md), (self.y - 1).rem_euclid(md)),
            2 => Vector2::new(self.x, (self.y - 1).rem_euclid(md)),
            3 => Vector2::new((self.x - 1).rem_euclid(md), self.y),
            4 => Vector2::new((self.x - 1).rem_euclid(md), (self.y + 1).rem_euclid(md)),
            5 => Vector2::new(self.x, (self.y + 1).rem_euclid(md)),
            _ => Vector2::new(self.x, self.y),
        }
    }

    fn centre(&self) -> Point2<f32> {
        let centre = int_to_float(&self);
        Point2::from(CENTRE * centre)
    }

    fn corner(&self, n: usize) -> Point2<f32> {
        let angle = ((n % 6) as f32) * TAU / 6.0;
        self.centre() + Vector2::new(angle.cos(), angle.sin())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn neighbour() {
        let org = Vector2::new(0, 0);
        assert_eq!(org.neighbour(0, 4), Vector2::new(1, 0));
        assert_eq!(org.neighbour(2, 4), Vector2::new(0, 3));
        assert_eq!(org.neighbour(3, 4), Vector2::new(3, 0));
        assert_eq!(org.neighbour(5, 4), Vector2::new(0, 1));
    }

    #[test]
    fn ambit() {
        let org = Vector2::new(0, 0);
        let amb = org.ambit(4);
        assert_eq!(amb.len(), 6);
        for j in 0..6 {
            assert_eq!(amb[j], org.neighbour(j, 4));
        }
    }

    #[test]
    fn centre() {
        assert_eq!(Vector2::new(1, 0).centre(), Point2::new(1.5, 0.8660254));
        assert_eq!(Vector2::new(0, 1).centre(), Point2::new(0.0, 1.7320508));
    }

    #[test]
    fn corners() {
        assert_eq!(Vector2::new(0, 0).corner(0), Point2::new(1.0, 0.0));
        assert_eq!(Vector2::new(0, 0).corners()[0], Point2::new(1.0, 0.0));
    }
}
