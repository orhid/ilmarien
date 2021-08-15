use crate::imaging::hexagonos::{Gon, PreGon};
use geo_types::Coordinate;
use log::{error, info};
use num_traits::identities::Zero;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use tiff::{decoder::*, encoder::*};

/* # math utils */

/// change a line point to a lattice point
fn enravel(j: usize, resolution: usize) -> Coordinate<i32> {
    Coordinate {
        x: (j / resolution) as i32,
        y: (j % resolution) as i32,
    }
}

/// change a lattice point to a line point
fn unravel(point: &Coordinate<i32>, resolution: usize) -> usize {
    (point.x.rem_euclid(resolution as i32) * resolution as i32
        + point.y.rem_euclid(resolution as i32)) as usize
}

/// find a grid point closest to given coordinate
fn find(point: &Coordinate<f64>, resolution: usize) -> Coordinate<i32> {
    Coordinate {
        x: resolution as f64 * point.x,
        y: resolution as f64 * point.y,
    }
    .find()
}

/// return a point in the unit square, regardless of resolution
fn cast(point: &Coordinate<i32>, resolution: usize) -> Coordinate<f64> {
    Coordinate {
        x: point.x as f64 / resolution as f64,
        y: point.y as f64 / resolution as f64,
    }
}

/* # branes */

#[derive(Clone)]
pub struct Brane<T>
where
    T: Copy,
{
    pub grid: Vec<T>,
    pub resolution: usize,
    pub variable: String,
}

impl<T: Copy> Brane<T> {
    /// read a value at given coordinate
    pub fn read(&self, point: &Coordinate<i32>) -> T {
        self.grid[self.unravel(&point)]
    }

    /// get a value nearest to given coordinate
    pub fn get(&self, point: &Coordinate<f64>) -> T {
        self.read(&self.find(point))
    }

    /// change a line point to a lattice point
    fn enravel(&self, j: usize) -> Coordinate<i32> {
        enravel(j, self.resolution)
    }

    /// change a lattice point to a line point
    fn unravel(&self, point: &Coordinate<i32>) -> usize {
        unravel(point, self.resolution)
    }

    /// find a grid point closest to given coordinate
    fn find(&self, point: &Coordinate<f64>) -> Coordinate<i32> {
        find(point, self.resolution)
    }

    /// return a point in the unit square, regardless of resolution
    fn cast(&self, point: &Coordinate<i32>) -> Coordinate<f64> {
        cast(point, self.resolution)
    }

    pub fn cell_count(&self) -> usize {
        3 * self.resolution * (self.resolution - 1) + 1
    }

    /// returns an area of a single cell as a fraction of the entire brane
    pub fn cell_area(&self) -> f64 {
        1.0 / self.cell_count() as f64
    }

    /// returns neighbouring points
    pub fn ambit(&self, point: &Coordinate<f64>) -> Vec<Coordinate<f64>> {
        self.find(point)
            .ambit(self.resolution as i32)
            .into_iter()
            .map(|gon| self.cast(&gon))
            .collect::<Vec<Coordinate<f64>>>()
    }

    /// returns neighbouring points
    pub fn exact_ambit(&self, point: &Coordinate<i32>) -> Vec<Coordinate<i32>> {
        point.ambit(self.resolution as i32)
    }

    /// produces an iterator over all coordinates in a brane of given resolution
    /// not necessarily an existing brane, could be used later to create a brane from a computation
    pub fn vec_iter(resolution: usize) -> std::vec::IntoIter<Coordinate<f64>> {
        (0..usize::pow(resolution, 2))
            .map(|j| cast(&enravel(j, resolution), resolution))
            .collect::<Vec<Coordinate<f64>>>()
            .into_iter()
    }

    /// produces a parallelised iterator over all coordinates in a brane of given resolution
    /// not necessarily an existing brane, could be used later to create a brane from a computation
    pub fn vec_par_iter(resolution: usize) -> rayon::vec::IntoIter<Coordinate<f64>> {
        (0..usize::pow(resolution, 2))
            .map(|j| cast(&enravel(j, resolution), resolution))
            .collect::<Vec<Coordinate<f64>>>()
            .into_par_iter()
    }

    /// produces an iterator over all exact coordinates in a brane
    /// used mainly for rendering
    pub fn exact_iter(&self) -> std::vec::IntoIter<Coordinate<i32>> {
        (0..usize::pow(self.resolution, 2))
            .map(|j| self.enravel(j))
            .collect::<Vec<Coordinate<i32>>>()
            .into_iter()
    }

    /// produces a parallelised iterator over all exact coordinates in a brane
    /// used mainly for rendering
    pub fn exact_par_iter(&self) -> rayon::vec::IntoIter<Coordinate<i32>> {
        (0..usize::pow(self.resolution, 2))
            .map(|j| self.enravel(j))
            .collect::<Vec<Coordinate<i32>>>()
            .into_par_iter()
    }
}

impl<T: Copy> IntoIterator for &Brane<T> {
    type Item = Coordinate<f64>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (0..usize::pow(self.resolution, 2))
            .map(|j| self.cast(&self.enravel(j)))
            .collect::<Vec<Coordinate<f64>>>()
            .into_iter()
    }
}

impl<T: Copy> IntoParallelIterator for &Brane<T> {
    type Item = Coordinate<f64>;
    type Iter = rayon::vec::IntoIter<Self::Item>;

    fn into_par_iter(self) -> Self::Iter {
        (0..usize::pow(self.resolution, 2))
            .map(|j| self.cast(&self.enravel(j)))
            .collect::<Vec<Coordinate<f64>>>()
            .into_par_iter()
    }
}

impl<T: Zero + Copy> Brane<T> {
    /// create a new brane filled with zeros
    pub fn zeros(resolution: usize) -> Self {
        info!("initialising empty brane at resolution {}", resolution);
        Brane {
            grid: vec![T::zero(); resolution.pow(2)],
            resolution,
            variable: "zeros".to_string(),
        }
    }
}

pub fn find_resolution(variable: &str) -> usize {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir("static") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with(variable) {
                    files.push(name);
                }
            }
        }
    }
    let mut resolutions = files
        .iter()
        .map(|file| {
            file[variable.len() + 1..file.len() - 4]
                .parse::<usize>()
                .expect("variable contains something weird")
        })
        .collect::<Vec<usize>>();
    resolutions.sort_unstable();
    resolutions
        .pop()
        .expect("found no brane for specified variable")
}

impl Brane<u8> {
    /// save brane to a .tif file
    pub fn save(&self) {
        let path_name = format!("static/{}-u8-{}.tif", self.variable, self.resolution);
        info!("saving brane to {}", path_name);
        TiffEncoder::new(&mut fs::File::create(&Path::new(&path_name)).unwrap())
            .unwrap()
            .write_image::<colortype::Gray8>(
                self.resolution as u32,
                self.resolution as u32,
                &self.grid,
            )
            .unwrap();
    }

    /// load brane with a given name from a .tif file
    pub fn load(variable: String) -> Self {
        let mut varextended = variable;
        varextended.push_str("-u8");
        let path_name = format!(
            "static/{}-{}.tif",
            varextended,
            find_resolution(&varextended)
        );
        info!("loading brane from {}", path_name);
        let mut file = fs::File::open(&Path::new(&path_name)).unwrap();
        let mut tiff = Decoder::new(&mut file).unwrap();

        Self::from(match tiff.read_image().unwrap() {
            DecodingResult::U8(vector) => vector,
            _ => panic!(),
        })
    }
}

impl Brane<u16> {
    /// save brane to a .tif file
    pub fn save(&self) {
        let path_name = format!("static/{}-u16-{}.tif", self.variable, self.resolution);
        info!("saving brane to {}", path_name);
        TiffEncoder::new(&mut fs::File::create(&Path::new(&path_name)).unwrap())
            .unwrap()
            .write_image::<colortype::Gray16>(
                self.resolution as u32,
                self.resolution as u32,
                &self.grid,
            )
            .unwrap();
    }

    /// load brane with a given name from a .tif file
    pub fn load(variable: String) -> Self {
        let mut varextended = variable;
        varextended.push_str("-u16");
        let path_name = format!(
            "static/{}-{}.tif",
            varextended,
            find_resolution(&varextended)
        );
        info!("loading brane from {}", path_name);
        let mut file = fs::File::open(&Path::new(&path_name)).unwrap();
        let mut tiff = Decoder::new(&mut file).unwrap();

        Self::from(match tiff.read_image().unwrap() {
            DecodingResult::U16(vector) => vector,
            _ => panic!(),
        })
    }
}

impl Brane<f64> {
    /// save brane to a .tif file
    pub fn save(&self) {
        Brane::<u16>::from(self).save();
    }

    /// load brane with a given name from a .tif file
    pub fn load(variable: String) -> Self {
        Self::from(&Brane::<u16>::load(variable))
    }
}

impl From<&Brane<f64>> for Brane<u16> {
    fn from(brane: &Brane<f64>) -> Self {
        Brane {
            grid: brane
                .grid
                .clone()
                .into_par_iter()
                .map(|value| (value * 2.0_f64.powi(16) - 1.0) as u16)
                .collect(),
            resolution: brane.resolution,
            variable: brane.variable.clone(),
        }
    }
}

impl From<&Brane<f64>> for Brane<u8> {
    fn from(brane: &Brane<f64>) -> Self {
        Brane {
            grid: brane
                .grid
                .clone()
                .into_par_iter()
                .map(|value| (value * 2.0_f64.powi(8) - 1.0) as u8)
                .collect(),
            resolution: brane.resolution,
            variable: brane.variable.clone(),
        }
    }
}

impl From<&Brane<u16>> for Brane<f64> {
    fn from(brane: &Brane<u16>) -> Self {
        Brane {
            grid: brane
                .grid
                .clone()
                .into_par_iter()
                .map(|value| value as f64 / (2.0_f64.powi(16) - 1.0))
                .collect(),
            resolution: brane.resolution,
            variable: brane.variable.clone(),
        }
    }
}

impl From<&Brane<u8>> for Brane<f64> {
    fn from(brane: &Brane<u8>) -> Self {
        Brane {
            grid: brane
                .grid
                .clone()
                .into_par_iter()
                .map(|value| value as f64 / (2.0_f64.powi(8) - 1.0))
                .collect(),
            resolution: brane.resolution,
            variable: brane.variable.clone(),
        }
    }
}

impl<T: Copy> From<Vec<T>> for Brane<T> {
    fn from(vector: Vec<T>) -> Self {
        let square = vector.len();
        let resolution = (square as f64).sqrt() as usize;
        if resolution.pow(2) == square {
            Brane {
                grid: vector,
                resolution,
                variable: "from-vec".to_string(),
            }
        } else {
            error!("cannot convert from unsquare vector");
            panic!("cannot convert from unsquare vector");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    /* # math utils */

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
    fn enravel_unravel() {
        assert_eq!(
            enravel(unravel(&Coordinate { x: 0, y: 1 }, 4), 4),
            Coordinate { x: 0, y: 1 }
        );
        assert_eq!(
            enravel(unravel(&Coordinate { x: 1, y: 0 }, 4), 4),
            Coordinate { x: 1, y: 0 }
        );
        assert_eq!(unravel(&enravel(1, 4), 4), 1);
        assert_eq!(unravel(&enravel(4, 4), 4), 4);
    }

    #[test]
    fn casting() {
        assert_eq!(
            cast(&Coordinate { x: 0, y: 0 }, 4),
            Coordinate { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            cast(&Coordinate { x: 0, y: 1 }, 4),
            Coordinate { x: 0.0, y: 0.25 }
        );
        assert_eq!(
            cast(&Coordinate { x: 1, y: 0 }, 4),
            Coordinate { x: 0.25, y: 0.0 }
        );
    }

    #[test]
    fn finding() {
        assert_eq!(
            find(&Coordinate { x: 0.0, y: 0.0 }, 4),
            Coordinate { x: 0, y: 0 }
        );
        assert_eq!(
            find(&Coordinate { x: 0.0, y: 0.25 }, 4),
            Coordinate { x: 0, y: 1 }
        );
        assert_eq!(
            find(&Coordinate { x: 0.25, y: 0.0 }, 4),
            Coordinate { x: 1, y: 0 }
        );
    }

    #[test]
    fn cast_find() {
        assert_eq!(
            find(&cast(&Coordinate { x: 0, y: 1 }, 4), 4),
            Coordinate { x: 0, y: 1 }
        );
        assert_eq!(
            find(&cast(&Coordinate { x: 1, y: 0 }, 4), 4),
            Coordinate { x: 1, y: 0 }
        );
        assert_eq!(
            cast(&find(&Coordinate { x: 0.0, y: 0.25 }, 4), 4),
            Coordinate { x: 0.0, y: 0.25 }
        );
        assert_eq!(
            cast(&find(&Coordinate { x: 0.25, y: 0.0 }, 4), 4),
            Coordinate { x: 0.25, y: 0.0 }
        );
    }

    /* # branes */

    #[test]
    fn brane_read() {
        let brane = Brane {
            grid: vec![0, 1, 2, 3],
            resolution: 2,
            variable: "test".to_string(),
        };
        assert_eq!(brane.read(&Coordinate { x: 1, y: 0 }), 2);
        assert_eq!(brane.read(&Coordinate { x: 0, y: 1 }), 1);
    }

    #[test]
    fn brane_get() {
        let brane = Brane {
            grid: vec![0, 1, 2, 3],
            resolution: 2,
            variable: "test".to_string(),
        };
        assert_eq!(brane.get(&Coordinate { x: 0.5, y: 0.0 }), 2);
        assert_eq!(brane.get(&Coordinate { x: 0.0, y: 0.5 }), 1);
    }

    #[test]
    fn brane_from_vec() {
        assert_eq!(Brane::from(vec![0, 1, 2, 3]).grid, vec![0, 1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn brane_from_unsquare_vec() {
        Brane::from(vec![0, 1, 2]);
    }

    #[test]
    fn brane_type_conversion() {
        let brane_f64 = Brane {
            grid: vec![0.0_f64, 0.5, 1.0, 0.5],
            resolution: 4,
            variable: "test".to_string(),
        };
        let brane_u16 = Brane {
            grid: vec![0_u16, 32767, 65535, 32767],
            resolution: 4,
            variable: "test".to_string(),
        };
        let brane_u8 = Brane {
            grid: vec![0_u8, 127, 255, 127],
            resolution: 4,
            variable: "test".to_string(),
        };
        assert_eq!(Brane::<u16>::from(&brane_f64).grid, brane_u16.grid);
        assert_eq!(Brane::<u8>::from(&brane_f64).grid, brane_u8.grid);
        assert_float_eq!(
            Brane::<f64>::from(&brane_u16).grid,
            brane_f64.grid,
            rmax <= vec![4.0 * EPSILON; 4]
        );
        assert_float_eq!(
            Brane::<f64>::from(&brane_u8).grid,
            brane_f64.grid,
            rmax <= vec![4.0 * EPSILON; 4]
        );
    }

    #[test]
    fn brane_zeros() {
        assert_eq!(Brane::<u8>::zeros(4).grid, vec![0; 16]);
        assert_eq!(Brane::<u16>::zeros(4).grid, vec![0; 16]);
        assert_float_eq!(
            Brane::<f64>::zeros(4).grid,
            vec![0.0; 16],
            abs <= vec![EPSILON; 16]
        );
    }

    #[test]
    fn brane_save_load() {
        let mut brane = Brane::<u8>::from(vec![0, 64, 128, 255]);
        brane.variable = "test-write".to_string();
        brane.save();
        assert!(Path::new("static/test-write-u8-2.tif").exists());
        assert_eq!(Brane::<u8>::load("test-write".to_string()).grid, brane.grid);

        let mut brane = Brane::<f64>::from(vec![0.0, 0.25, 0.5, 0.75]);
        brane.variable = "test-write".to_string();
        brane.save();
        assert!(Path::new("static/test-write-u16-2.tif").exists());
        assert_float_eq!(
            Brane::<f64>::load("test-write".to_string()).grid,
            brane.grid,
            rmax <= vec![EPSILON; 4]
        );

        fs::remove_file("static/test-write-u8-2.tif").expect("test failed");
        fs::remove_file("static/test-write-u16-2.tif").expect("test failed");
    }
}
