use geo::Coordinate;
use std::fs::File;
use std::path::Path;
use tiff::{decoder::*, encoder::*};

/* branes */

pub struct Brane {
    grid: Vec<u16>,
    pub variable: String,
    pub resolution: usize,
}

impl Brane {
    pub fn save(&self) {
        let path_name = format!("static/{}-{}.tif", self.variable, self.resolution);
        TiffEncoder::new(&mut File::create(&Path::new(&path_name)).unwrap())
            .unwrap()
            .write_image::<colortype::Gray16>(
                self.resolution as u32,
                self.resolution as u32,
                &self.grid,
            )
            .unwrap();
    }

    pub fn insert(&mut self, point: &Coordinate<i32>, value: f64) {
        // should panic if value not in [0,1]
        self.grid[unravel(&point, self.resolution)] = encode(value);
    }

    pub fn get(&self, point: &Coordinate<i32>) -> f64 {
        decode(self.grid[unravel(point, self.resolution)])
    }
}

impl IntoIterator for &Brane {
    type Item = Coordinate<i32>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (0..usize::pow(self.resolution as usize, 2))
            .map(|j| enravel(j, self.resolution))
            .collect::<Vec<Coordinate<i32>>>()
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

fn enravel(j: usize, resolution: usize) -> Coordinate<i32> {
    //! change a line point to a lattice point
    Coordinate {
        x: (j / resolution) as i32,
        y: (j % resolution) as i32,
    }
}

fn unravel(point: &Coordinate<i32>, resolution: usize) -> usize {
    //! change a lattice point to a line point
    (point.x * resolution as i32 + point.y) as usize
}

pub fn new(variable: String, resolution: usize) -> Brane {
    Brane {
        grid: (0..usize::pow(resolution, 2)).map(|_| 0).collect(),
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
        resolution: tiff.dimensions().unwrap().0 as usize,
    }
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
        assert_eq!(enravel(0, 4), Coordinate { x: 0, y: 0 });
        assert_eq!(enravel(1, 4), Coordinate { x: 0, y: 1 });
        assert_eq!(enravel(4, 4), Coordinate { x: 1, y: 0 });
        assert_eq!(enravel(7, 4), Coordinate { x: 1, y: 3 });
    }

    #[test]
    fn unravelling() {
        assert_eq!(unravel(&Coordinate { x: 0, y: 0 }, 4), 0);
        assert_eq!(unravel(&Coordinate { x: 0, y: 1 }, 4), 1);
        assert_eq!(unravel(&Coordinate { x: 1, y: 0 }, 4), 4);
        assert_eq!(unravel(&Coordinate { x: 1, y: 3 }, 4), 7);
    }

    #[test]
    fn create_and_insert_into_brane() {
        let mut brane = new("test".to_string(), 4);
        let point = Coordinate { x: 0, y: 1 };
        assert_eq!(brane.get(&point), 0.0);
        brane.insert(&point, 1.0);
        assert_eq!(brane.get(&point), 1.0);
    }

    #[test]
    fn initialise_and_save_brane() {
        let grid: Vec<u16> = (0..16).map(|j| j * u16::pow(2, 12)).collect();
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
}
