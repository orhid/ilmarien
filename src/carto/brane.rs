use crate::carto::{
    datum::{DatumRe, DatumZa},
    honeycomb::HoneyCellToroidal,
};
use log::{error, info};
use num_traits::identities::Zero;
use rayon::prelude::*;
use std::{fs, path::Path};
use tiff::{decoder::*, encoder::*};

/* # branes */

#[derive(Clone)]
pub struct Brane<T> {
    pub grid: Vec<T>,
    pub resolution: usize,
    pub variable: String,
}

impl<T> Brane<T> {
    /// find a grid datum closest to given coordinate
    pub fn find(&self, datum: &DatumRe) -> DatumZa {
        datum.find(self.resolution)
    }

    /// return a datum in the unit square, regardless of resolution
    pub fn cast(&self, datum: &DatumZa) -> DatumRe {
        datum.cast(self.resolution)
    }

    /// returns neighbouring datums
    pub fn ambit(&self, datum: &DatumRe) -> Vec<DatumRe> {
        datum
            .find(self.resolution)
            .ambit_toroidal(self.resolution as i32)
            .into_iter()
            .map(|gon| gon.cast(self.resolution))
            .collect::<Vec<DatumRe>>()
    }

    /// returns neighbouring datums
    pub fn ambit_exact(&self, datum: &DatumZa) -> Vec<DatumZa> {
        datum.ambit_toroidal(self.resolution as i32)
    }

    /// produces an iterator over all coordinates in a brane of given resolution
    /// not necessarily an existing brane, could be used later to create a brane from a computation
    pub fn iter(resolution: usize) -> std::vec::IntoIter<DatumRe> {
        (0..resolution.pow(2))
            .map(|j| DatumZa::enravel(j, resolution).cast(resolution))
            .collect::<Vec<DatumRe>>()
            .into_iter()
    }

    /// produces a parallelised iterator over all coordinates in a brane of given resolution
    /// not necessarily an existing brane, could be used later to create a brane from a computation
    pub fn par_iter(resolution: usize) -> rayon::vec::IntoIter<DatumRe> {
        (0..resolution.pow(2))
            .map(|j| DatumZa::enravel(j, resolution).cast(resolution))
            .collect::<Vec<DatumRe>>()
            .into_par_iter()
    }

    /// produces an iterator over all exact coordinates in a brane
    /// used mainly for rendering
    pub fn iter_exact(&self) -> std::vec::IntoIter<DatumZa> {
        (0..self.resolution.pow(2))
            .map(|j| DatumZa::enravel(j, self.resolution))
            .collect::<Vec<DatumZa>>()
            .into_iter()
    }

    /// produces a parallelised iterator over all exact coordinates in a brane
    /// used mainly for rendering
    pub fn par_iter_exact(&self) -> rayon::vec::IntoIter<DatumZa> {
        (0..self.resolution.pow(2))
            .map(|j| DatumZa::enravel(j, self.resolution))
            .collect::<Vec<DatumZa>>()
            .into_par_iter()
    }
}

impl<T> IntoIterator for &Brane<T> {
    type Item = DatumRe;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Brane::<T>::iter(self.resolution)
    }
}

impl<T> IntoParallelIterator for &Brane<T> {
    type Item = DatumRe;
    type Iter = rayon::vec::IntoIter<Self::Item>;

    fn into_par_iter(self) -> Self::Iter {
        Brane::<T>::par_iter(self.resolution)
    }
}

impl<T: Copy> Brane<T> {
    /// read a value at given coordinate
    pub fn read(&self, datum: &DatumZa) -> T {
        self.grid[datum.unravel(self.resolution)]
    }

    pub fn insert(&mut self, datum: &DatumZa, value: T) {
        self.grid[datum.unravel(self.resolution)] = value;
    }

    /// get a value nearest to given coordinate
    pub fn get(&self, datum: &DatumRe) -> T {
        self.read(&datum.find(self.resolution))
    }
}

impl<T: Zero + Clone> Brane<T> {
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

fn find_resolution(variable: &str) -> usize {
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
            find_resolution(&varextended),
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
            find_resolution(&varextended),
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

    /// print out basic statistical information
    pub fn stats(&self) {
        println!("stats for {}", self.variable);
        println!(
            "min {}",
            self.grid
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        );
        println!(
            "med {}",
            self.grid.iter().sum::<f64>() / self.grid.len() as f64
        );
        println!(
            "max {}",
            self.grid
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        );
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

impl<T> From<Vec<T>> for Brane<T> {
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

/* ## onions */

pub type Onion<T> = Brane<Vec<T>>;

impl<T> Onion<T> {
    pub fn push(&mut self, datum: &DatumZa, value: T) {
        self.grid[datum.unravel(self.resolution)].push(value);
    }
}

impl<T: Clone> Onion<T> {
    /// creates iterator over column at given datum
    pub fn iter_column(&self, datum: &DatumZa) -> std::vec::IntoIter<T> {
        self.grid[datum.unravel(self.resolution)]
            .clone()
            .into_iter()
    }

    pub fn top(&self, datum: &DatumZa) -> Option<T> {
        self.grid[datum.unravel(self.resolution)].clone().pop()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    /* # branes */

    #[test]
    fn brane_read() {
        let brane = Brane {
            grid: vec![0, 1, 2, 3],
            resolution: 2,
            variable: "test".to_string(),
        };
        assert_eq!(brane.read(&DatumZa { x: 1, y: 0 }), 2);
        assert_eq!(brane.read(&DatumZa { x: 0, y: 1 }), 1);
    }

    #[test]
    fn brane_get() {
        let brane = Brane {
            grid: vec![0, 1, 2, 3],
            resolution: 2,
            variable: "test".to_string(),
        };
        assert_eq!(brane.get(&DatumRe { x: 0.5, y: 0.0 }), 2);
        assert_eq!(brane.get(&DatumRe { x: 0.0, y: 0.5 }), 1);
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

    /* ## onions */

    #[test]
    fn top_does_not_pop() {
        let onion = Onion::from(vec![vec![0, 1], vec![0, 1], vec![0, 1], vec![0, 1]]);
        let datum = DatumZa::new(0, 0);
        assert_eq!(onion.top(&datum), Some(1));
        assert_eq!(onion.grid[0].len(), 2);
    }
}
