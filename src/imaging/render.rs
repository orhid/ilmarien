use crate::imaging::{
    cartography::Brane,
    colour::Ink,
    hexagonos::{Gon, Tile, Tileable},
};
use geo_types::{Coordinate, LineString, Polygon};
use svg::node::element::Path;

trait ToSVG {
    fn svg(&self) -> Path;
}

impl ToSVG for Polygon<f64> {
    fn svg(&self) -> Path {
        use std::fmt::Write;
        let mut path = String::new();
        for contour in std::iter::once(self.exterior()).chain(self.interiors().iter()) {
            let mut points = contour.points_iter();
            if let Some(first_point) = points.next() {
                write!(path, "M {:?} {:?}", first_point.x(), first_point.y()).unwrap()
            }
            for point in points {
                write!(path, " L {:?} {:?}", point.x(), point.y()).unwrap();
            }
            write!(path, " Z ").unwrap();
        }

        Path::new().set("fill-rule", "evenodd").set("d", path)
    }
}

pub trait Renderable {
    fn render<T>(&self, ink: T)
    where
        T: Ink;

    /*
    fn render_triple<T>(&self, ink: T)
    where
        T: Ink;
    */
}

impl Renderable for Brane {
    fn render<T>(&self, ink: T)
    where
        T: Ink,
    {
        let one: i32 = self.resolution as i32;
        let mut image = svg::Document::new().set("viewBox", (-one, -one, 2 * one, 2 * one));

        for point in self {
            let tiling: Coordinate<i32> = match point.tile(one) {
                Tile::Y => Coordinate { x: 0, y: 0 },
                Tile::R => Coordinate { x: 0, y: -one },
                Tile::B => Coordinate { x: -one, y: 0 },
                Tile::G => Coordinate { x: -one, y: -one },
            };

            image = image.add(
                Polygon::new(LineString::from((point + tiling).corners()), vec![])
                    .svg()
                    .set("fill", ink.paint(self.get(&point))),
            );
        }
        let path_name = format!("bounce/{}-{}.svg", self.variable, self.resolution);
        svg::save(&path_name, &image).unwrap();
    }

    /*
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
    */
}
