use crate::imaging::{
    cartography::Brane,
    colour::Ink,
    hexagonos::{Gon, Tile, Tileable},
};
use geo::Coordinate;
use svg::node::element::Polygon;

fn svg_hexagon(corners: Vec<Coordinate<f64>>) -> Polygon {
    Polygon::new().set(
        "points",
        format!(
            "{},{} {},{} {},{} {},{} {},{} {},{}",
            corners[0].x,
            corners[0].y,
            corners[1].x,
            corners[1].y,
            corners[2].x,
            corners[2].y,
            corners[3].x,
            corners[3].y,
            corners[4].x,
            corners[4].y,
            corners[5].x,
            corners[5].y,
        ),
    )
}

pub trait Renderable {
    fn render<T>(&self, ink: T)
    where
        T: Ink;

    fn render_triple<T>(&self, ink: T)
    where
        T: Ink;
}

impl Renderable for Brane {
    fn render<T>(&self, ink: T)
    where
        T: Ink,
    {
        let one: i32 = self.resolution as i32;
        let mut image = svg::Document::new().set("viewBox", (-one, -one, 2 * one, 2 * one));
        for point in self {
            let tiling: Coordinate<i32> = match point.tile(self.resolution as i32) {
                Tile::Y => Coordinate { x: 0, y: 0 },
                Tile::R => Coordinate { x: 0, y: -one },
                Tile::B => Coordinate { x: -one, y: 0 },
                Tile::G => Coordinate { x: -one, y: -one },
            };
            let hexagon =
                svg_hexagon((point + tiling).corners()).set("fill", ink.paint(self.get(&point)));
            image = image.add(hexagon);
        }
        let path_name = format!("bounce/{}-{}.svg", self.variable, self.resolution);
        svg::save(&path_name, &image).unwrap();
    }

    fn render_triple<T>(&self, ink: T)
    where
        T: Ink,
    {
        let one: i32 = self.resolution as i32;
        let mut image = svg::Document::new().set(
            "viewBox",
            (-one as f32 * 1.25, -one as f32 * 1.125, 4 * one, 4 * one),
        );
        for point in self {
            let tiling = match point.tile(self.resolution as i32) {
                Tile::Y => vec![
                    Coordinate { x: 0, y: 0 },
                    Coordinate { x: one, y: 0 },
                    Coordinate { x: 0, y: one },
                ],
                Tile::R => vec![
                    Coordinate { x: 0, y: -one },
                    Coordinate { x: 0, y: 0 },
                    Coordinate { x: one, y: -one },
                ],
                Tile::B => vec![
                    Coordinate { x: -one, y: 0 },
                    Coordinate { x: 0, y: 0 },
                    Coordinate { x: -one, y: one },
                ],
                Tile::G => vec![
                    Coordinate { x: -one, y: -one },
                    Coordinate { x: -one, y: 0 },
                    Coordinate { x: 0, y: -one },
                ],
            };
            for tile in tiling {
                let hexagon =
                    svg_hexagon((point + tile).corners()).set("fill", ink.paint(self.get(&point)));
                image = image.add(hexagon);
            }
        }
        let path_name = format!("bounce/{}-{}-tri.svg", self.variable, self.resolution);
        svg::save(&path_name, &image).unwrap();
    }
}
