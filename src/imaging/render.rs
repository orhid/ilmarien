use crate::imaging::{
    cartography::Brane,
    colour::Ink,
    hexagonos::{Gon, Tile, Tileable},
};
use geo::orient::{Direction, Orient};
use geo_booleanop::boolean::BooleanOp;
use geo_types::{Coordinate, LineString, MultiPolygon, Polygon};
use std::collections::{HashMap, VecDeque};
use svg::node::element::Path;

trait ToSVG {
    fn svg(&self) -> Path;
}

fn _poly_to_svg(poly: &Polygon<f64>) -> String {
    if poly.exterior().0.is_empty() {
        "".into()
    } else {
        format!("M{}", _poly_rings_to_svg(poly))
    }
}

fn _poly_rings_to_svg(poly: &Polygon<f64>) -> String {
    let mut lines: Vec<LineString<f64>> = poly.interiors().into();
    let exterior: &LineString<f64> = poly.exterior();
    lines.insert(0, exterior.clone());

    lines
        .iter()
        .map(|l| _poly_ring_to_svg(&l))
        .collect::<Vec<String>>()
        .join("M")
}

fn _poly_ring_to_svg(line: &LineString<f64>) -> String {
    line.0
        .iter()
        .map(|c| _coord_to_svg(&c))
        .collect::<Vec<String>>()
        .join("L")
}

fn _coord_to_svg(coord: &Coordinate<f64>) -> String {
    format!("{} {}", coord.x, coord.y)
}

impl ToSVG for Polygon<f64> {
    fn svg(&self) -> Path {
        Path::new()
            .set("d", _poly_to_svg(self))
            .set("fill-rule", "evenodd")
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

fn cascade(mut terrace: VecDeque<MultiPolygon<f64>>) -> MultiPolygon<f64> {
    // this can be less naive
    while terrace.len() > 1 {
        let polya = terrace.pop_front().unwrap();
        let polyb = terrace.pop_front().unwrap();
        terrace.push_back(polya.union(&polyb));
    }
    terrace.pop_front().unwrap()
}

impl Renderable for Brane {
    fn render<T>(&self, ink: T)
    where
        T: Ink,
    {
        let one: i32 = self.resolution as i32;
        let mut terraces = HashMap::new();
        for point in self {
            let tiling: Coordinate<i32> = match point.tile(one) {
                Tile::Y => Coordinate { x: 0, y: 0 },
                Tile::R => Coordinate { x: 0, y: -one },
                Tile::B => Coordinate { x: -one, y: 0 },
                Tile::G => Coordinate { x: -one, y: -one },
            };
            terraces
                .entry(ink.paint(self.get(&point)))
                .or_insert(VecDeque::<MultiPolygon<f64>>::new())
                .push_back(MultiPolygon::from(vec![Polygon::new(
                    LineString::from((point + tiling).corners()),
                    vec![],
                )]));
        }

        let mut image = svg::Document::new().set("viewBox", (-one, -one, 2 * one, 2 * one));
        for (paint, terrace) in terraces {
            let multigon = cascade(terrace);
            for polygon in multigon {
                // orienting should be done in union function and only for polygons with interiors
                // others will work fine even if they are in the wrong orientation
                // this might fuck with areas later though, so maybe orient all
                image = image.add(
                    polygon
                        .orient(Direction::Default)
                        .svg()
                        .set("fill", paint.as_str()),
                );
            }
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
