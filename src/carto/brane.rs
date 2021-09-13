use crate::carto::{
    datum::{DatumRe, DatumZa},
    honeycomb::HoneyCellToroidal,
};
use log::{error, trace};
use num_traits::{identities::Zero, MulAdd};
use rayon::prelude::*;
use std::{
    fs,
    iter::FromIterator,
    ops::{Add, Div, Mul, Sub},
    path::Path,
};
use tiff::{decoder::*, encoder::*};

/* # branes */

#[derive(Clone)]
pub struct Brane<T> {
    pub grid: Vec<T>,
    pub resolution: usize,
    pub variable: String,
}

impl<T> Brane<T> {
    /// change value at given datum
    pub fn insert(&mut self, datum: &DatumZa, value: T) {
        self.grid[datum.unravel(self.resolution)] = value;
    }

    /// find a grid datum closest to given coordinate
    pub fn find(&self, datum: &DatumRe) -> DatumZa {
        datum.floor(self.resolution)
    }

    /// find a grid datum closest to given coordinate
    pub fn find_accurate(&self, datum: &DatumRe) -> DatumZa {
        datum.find(self.resolution)
    }

    /// return a datum in the unit square, regardless of resolution
    pub fn cast(&self, datum: &DatumZa) -> DatumRe {
        datum.cast(self.resolution)
    }

    /// returns neighbouring datums
    pub fn ambit(&self, datum: &DatumRe) -> [DatumRe; 6] {
        datum
            .floor(self.resolution)
            .ambit_toroidal(self.resolution as i32)
            .map(|gon| gon.cast(self.resolution))
    }

    /// returns neighbouring datums
    pub fn ambit_exact(&self, datum: &DatumZa) -> [DatumZa; 6] {
        datum.ambit_toroidal(self.resolution as i32)
    }
}

impl<T: Clone> Brane<T> {
    /// read a value at given coordinate
    pub fn read(&self, datum: &DatumZa) -> T {
        self.grid[datum.unravel(self.resolution)].clone()
    }

    /// get a value nearest to given coordinate
    pub fn get(&self, datum: &DatumRe) -> T {
        self.read(&datum.floor(self.resolution))
    }
}

impl<T: Zero + Clone> Brane<T> {
    /// create a new brane filled with zeros
    pub fn zeros(resolution: usize) -> Self {
        Brane {
            grid: vec![T::zero(); resolution.pow(2)],
            resolution,
            variable: "zeros".to_string(),
        }
    }
}

macro_rules! impl_op_internal {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: $trait> $trait for Brane<T>
        where
            Vec<T>: FromIterator<<T as $trait>::Output>,
        {
            type Output = Self;

            fn $method(self, other: Self) -> Self {
                Self {
                    grid: self
                        .grid
                        .into_iter()
                        .zip(other.grid.into_iter())
                        .map(|(x, y)| x $op y)
                        .collect::<Vec<T>>(),
                    resolution: self.resolution,
                    variable: format!("op-{}-{}", self.variable, other.variable),
                }
            }
        }
    };
}

macro_rules! impl_op_external {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: $trait + Copy> $trait<T> for Brane<T>
        where
            Vec<T>: FromIterator<<T as $trait>::Output>,
        {
            type Output = Self;

            fn $method(self, other: T) -> Self {
                Self {
                    grid: self
                        .grid
                        .into_iter()
                        .map(|x| x $op other)
                        .collect::<Vec<T>>(),
                    resolution: self.resolution,
                    variable: format!("op-{}", self.variable),
                }
            }
        }
    };
}

impl_op_internal!(Add, add, +);
impl_op_internal!(Sub, sub, -);
impl_op_external!(Add, add, +);
impl_op_external!(Sub, sub, -);
impl_op_external!(Mul, mul, *);
impl_op_external!(Div, div, /);

impl<T: MulAdd + Copy> Brane<T>
where
    Vec<T>: FromIterator<<T as MulAdd>::Output>,
{
    pub fn mul_add(self, xmul: T, xadd: T) -> Self {
        Self {
            grid: self
                .grid
                .into_iter()
                .map(|x| x.mul_add(xmul, xadd))
                .collect::<Vec<T>>(),
            resolution: self.resolution,
            variable: format!("muladd-{}", self.variable),
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

// saving and loading could probably be refactored with a macro

impl Brane<u8> {
    /// save brane to a .tif file
    pub fn save(&self) {
        let path_name = format!("static/{}-u8-{}.tif", self.variable, self.resolution);
        trace!("saving brane to {}", path_name);
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
        trace!("loading brane from {}", path_name);
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
        trace!("saving brane to {}", path_name);
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
        trace!("loading brane from {}", path_name);
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
            "min: {}",
            self.grid
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        );
        println!(
            "med: {}",
            self.grid.iter().sum::<f64>() / self.grid.len() as f64
        );
        println!(
            "max: {}",
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

impl<T: PartialOrd> Onion<T> {
    pub fn sort_columns(&mut self) {
        for column in &mut self.grid {
            column.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
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
    fn brane_add_self() {
        assert_eq!(
            (Brane::from(vec![0, 1, 2, 3]) + Brane::from(vec![1, 2, 3, 4])).grid,
            Brane::from(vec![1, 3, 5, 7]).grid
        );
    }

    #[test]
    fn brane_sub_self() {
        assert_eq!(
            (Brane::from(vec![1, 2, 3, 4]) - Brane::from(vec![0, 1, 2, 3])).grid,
            Brane::from(vec![1, 1, 1, 1]).grid
        );
    }

    #[test]
    fn brane_add() {
        assert_eq!(
            (Brane::from(vec![1, 2, 3, 4]) + 2).grid,
            Brane::from(vec![3, 4, 5, 6]).grid
        );
    }

    #[test]
    fn brane_sub() {
        assert_eq!(
            (Brane::from(vec![1, 2, 3, 4]) - 1).grid,
            Brane::from(vec![0, 1, 2, 3]).grid
        );
    }

    #[test]
    fn brane_mul() {
        assert_eq!(
            (Brane::from(vec![1, 2, 3, 4]) * 2).grid,
            Brane::from(vec![2, 4, 6, 8]).grid
        );
    }

    #[test]
    fn brane_div() {
        assert_eq!(
            (Brane::from(vec![2, 4, 6, 8]) / 2).grid,
            Brane::from(vec![1, 2, 3, 4]).grid
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
    fn onion_push() {
        let mut onion = Onion::from(vec![vec![0, 1]]);
        onion.push(&DatumZa::new(0, 0), 2);
        assert_eq!(onion.grid[0], vec![0, 1, 2]);
    }

    #[test]
    fn onion_sort_columns() {
        let mut onion = Onion::from(vec![vec![1, 0]]);
        onion.sort_columns();
        assert_eq!(onion.grid[0], vec![0, 1]);
    }

    #[test]
    fn onion_top_does_not_pop() {
        let onion = Onion::from(vec![vec![0, 1], vec![0, 1], vec![0, 1], vec![0, 1]]);
        let datum = DatumZa::new(0, 0);
        assert_eq!(onion.top(&datum), Some(1));
        assert_eq!(onion.grid[0].len(), 2);
    }
}
