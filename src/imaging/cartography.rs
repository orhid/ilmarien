use crate::imaging::{colour::Ink, hexagonos::Gon};
use nalgebra::{Point2, Vector2};
use std::fs::File;
use std::path::Path;
//use svg::node::element::path::Data;
use svg::node::element::Polygon;
use tiff::{decoder::*, encoder::*};

/* branes */

pub struct Brane {
    grid: Vec<u16>,
    variable: String,
    pub resolution: u32,
}

impl Brane {
    pub fn render<T>(&self, ink: T)
    where
        T: Ink,
    {
        let one: i32 = self.resolution as i32;
        let mut image = svg::Document::new().set("viewBox", (-one, -one, 2 * one, 2 * one));
        for point in self {
            let tiling: Vector2<i32> = match in_tile(&point, self.resolution) {
                Tile::Y => Vector2::new(0, 0),
                Tile::R => Vector2::new(0, -one),
                Tile::B => Vector2::new(-one, 0),
                Tile::G => Vector2::new(-one, -one),
            };
            let hexagon =
                svg_hexagon((point + tiling).corners()).set("fill", ink.paint(self.get(&point)));
            image = image.add(hexagon);
        }
        let path_name = format!("bounce/{}-{}.svg", self.variable, self.resolution);
        svg::save(&path_name, &image).unwrap();
    }

    pub fn render_triple<T>(&self, ink: T)
    where
        T: Ink,
    {
        let one: i32 = self.resolution as i32;
        let mut image = svg::Document::new().set(
            "viewBox",
            (-one as f32 * 1.25, -one as f32 * 1.125, 4 * one, 4 * one),
        );
        for point in self {
            let tiling = match in_tile(&point, self.resolution) {
                Tile::Y => vec![
                    Vector2::new(0, 0),
                    Vector2::new(one, 0),
                    Vector2::new(0, one),
                ],
                Tile::R => vec![
                    Vector2::new(0, -one),
                    Vector2::new(0, 0),
                    Vector2::new(one, -one),
                ],
                Tile::B => vec![
                    Vector2::new(-one, 0),
                    Vector2::new(0, 0),
                    Vector2::new(-one, one),
                ],
                Tile::G => vec![
                    Vector2::new(-one, -one),
                    Vector2::new(-one, 0),
                    Vector2::new(0, -one),
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

    pub fn save(&self) {
        let path_name = format!("static/{}-{}.tif", self.variable, self.resolution);
        TiffEncoder::new(&mut File::create(&Path::new(&path_name)).unwrap())
            .unwrap()
            .write_image::<colortype::Gray16>(self.resolution, self.resolution, &self.grid)
            .unwrap();
    }

    pub fn insert(&mut self, vector: &Vector2<i32>, value: f64) {
        // should panic if value not in [0,1]
        self.grid[unravel(&vector, self.resolution)] = encode(value);
    }

    fn get(&self, vector: &Vector2<i32>) -> f64 {
        decode(self.grid[unravel(vector, self.resolution)])
    }
}

impl IntoIterator for &Brane {
    type Item = Vector2<i32>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (0..usize::pow(self.resolution as usize, 2))
            .map(|j| enravel(j, self.resolution))
            .collect::<Vec<Vector2<i32>>>()
            .into_iter()
    }
}

fn encode(value: f64) -> u16 {
    //! encode a float from the [0,1] interval to u16 bit range
    (value * 65535.0) as u16
}

fn decode(value: u16) -> f64 {
    //! encode a u16 bit into the [0,1] interval
    value as f64 / 65535.0
}

fn enravel(x: usize, resolution: u32) -> Vector2<i32> {
    //! change a line point to a lattice point
    Vector2::new(
        (x / resolution as usize) as i32,
        (x % resolution as usize) as i32,
    )
}

fn unravel(vector: &Vector2<i32>, resolution: u32) -> usize {
    //! change a lattice point to a line point
    (vector.x * resolution as i32 + vector.y) as usize
}

pub fn new(variable: String, resolution: u32) -> Brane {
    Brane {
        grid: (0..u32::pow(resolution, 2)).map(|_| 0).collect(),
        variable: variable,
        resolution: resolution,
    }
}

pub fn load(variable: String) -> Brane {
    // TODO : open file with best avaliable resolution

    let path_name = format!("static/{}.tif", variable);
    let mut file = File::open(&Path::new(&path_name)).unwrap();
    let mut tiff = Decoder::new(&mut file).unwrap();
    Brane {
        grid: match tiff.read_image().unwrap() {
            DecodingResult::U16(vector) => vector,
            _ => panic!(), // one may want to implement more types in the future
        },
        variable: variable,
        resolution: tiff.dimensions().unwrap().0,
    }
}

/* hexagon tiling */

#[derive(Debug, PartialEq)]
enum Tile {
    R,
    G,
    B,
    Y,
}

fn in_tile(vector: &Vector2<i32>, resolution: u32) -> Tile {
    let one: i32 = resolution as i32;
    if 2 * vector.x + vector.y - one <= 0 {
        if vector.x + 2 * vector.y - one < 0 {
            Tile::Y
        } else {
            Tile::R
        }
    } else {
        if 2 * vector.x + vector.y - 2 * one <= 0 {
            if vector.x - vector.y <= 0 {
                Tile::R
            } else {
                Tile::B
            }
        } else {
            if vector.x + 2 * vector.y - 2 * one < 0 {
                Tile::B
            } else {
                Tile::G
            }
        }
    }
}

/* svg rendergin */

fn svg_hexagon(corners: Vec<Point2<f32>>) -> Polygon {
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

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0001;

    #[test]
    fn encoding() {
        assert_eq!(encode(0.0), 0);
        assert_eq!(encode(0.5), 32767);
        assert_eq!(encode(1.0), 65535);
    }

    #[test]
    fn decoding() {
        assert_float_eq!(decode(0), 0.0, abs <= EPSILON);
        assert_float_eq!(decode(32767), 0.5, abs <= EPSILON);
        assert_float_eq!(decode(65535), 1.0, abs <= f64::EPSILON);
    }

    #[test]
    fn enravelling() {
        assert_eq!(enravel(0, 4), Vector2::new(0, 0));
        assert_eq!(enravel(1, 4), Vector2::new(0, 1));
        assert_eq!(enravel(4, 4), Vector2::new(1, 0));
        assert_eq!(enravel(7, 4), Vector2::new(1, 3));
    }

    #[test]
    fn unravelling() {
        assert_eq!(unravel(&Vector2::new(0, 0), 4), 0);
        assert_eq!(unravel(&Vector2::new(0, 1), 4), 1);
        assert_eq!(unravel(&Vector2::new(1, 0), 4), 4);
        assert_eq!(unravel(&Vector2::new(1, 3), 4), 7);
    }

    #[test]
    fn create_and_insert_into_brane() {
        let mut brane = new("test".to_string(), 4);
        let point = Vector2::new(0, 1);
        assert_eq!(brane.get(&point), 0.0);
        brane.insert(&point, 1.0);
        assert_eq!(brane.get(&point), 1.0);
    }

    #[test]
    fn initialise_and_save_brane() {
        let grid: Vec<u16> = (0..16).map(|x| x * u16::pow(2, 12)).collect();
        let brane = Brane {
            grid: grid,
            variable: "test/write".to_string(),
            resolution: 4,
        };
        brane.save();
        assert!(Path::new("static/test/write-4.tif").exists());
    }

    #[test]
    fn load_brane() {
        let brane = load("test/read-4".to_string());
        assert_eq!(brane.resolution, 4);
    }

    #[test]
    fn tile_placement() {
        assert_eq!(in_tile(&Vector2::new(0, 0), 4), Tile::Y);
        assert_eq!(in_tile(&Vector2::new(1, 1), 4), Tile::Y);
        assert_eq!(in_tile(&Vector2::new(3, 1), 4), Tile::B);
        assert_eq!(in_tile(&Vector2::new(1, 3), 4), Tile::R);
        assert_eq!(in_tile(&Vector2::new(3, 3), 4), Tile::G);
        assert_eq!(in_tile(&Vector2::new(4, 4), 4), Tile::G);
    }
}
