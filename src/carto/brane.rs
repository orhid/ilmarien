use crate::{
    carto::datum::{DatumRe, DatumZa},
    units::Unit,
};
use log::trace;
use rayon::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::{fs, path::Path};
use tiff::{decoder::*, encoder::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolution(usize);

impl Resolution {
    pub const fn confine(value: usize) -> Self {
        Self(value)
    }

    pub fn release(self) -> usize {
        self.0
    }

    pub fn square(self) -> usize {
        self.0.pow(2)
    }
}

macro_rules! impl_from_resolution {
    ($num: ty) => {
        impl From<Resolution> for $num {
            fn from(res: Resolution) -> $num {
                res.0 as $num
            }
        }
    };
}

impl_from_resolution!(usize);
impl_from_resolution!(u32);
impl_from_resolution!(i32);
impl_from_resolution!(f64);

/* # branes */

#[derive(Clone)]
pub struct Brane<T> {
    pub grid: Vec<T>,
    pub resolution: Resolution,
}

impl<T: Send> Brane<T> {
    pub fn new(grid: Vec<T>, resolution: Resolution) -> Self {
        Self { grid, resolution }
    }

    /* ## operations */

    pub fn create_by_index<F>(resolution: Resolution, f: F) -> Self
    where
        F: Fn(usize) -> T + Sync + Send,
    {
        Self::new(
            (0..resolution.square())
                .into_par_iter()
                .map(f)
                .collect::<Vec<T>>(),
            resolution,
        )
    }

    pub fn create_by_datum<F>(resolution: Resolution, f: F) -> Self
    where
        F: Fn(DatumRe) -> T + Sync + Send,
    {
        Self::new(
            (0..resolution.square())
                .into_par_iter()
                .map(|j| f(DatumZa::enravel(j, resolution).cast(resolution)))
                .collect::<Vec<T>>(),
            resolution,
        )
    }

    pub fn operate_by_index<F, S>(&self, f: F) -> Brane<S>
    where
        S: Send,
        F: Fn(usize) -> S + Sync + Send,
    {
        Brane::new(
            (0..self.resolution.square())
                .into_par_iter()
                .map(f)
                .collect::<Vec<S>>(),
            self.resolution,
        )
    }

    pub fn operate_by_value<F, S>(self, f: F) -> Brane<S>
    where
        S: Send,
        F: Fn(T) -> S + Sync + Send,
    {
        Brane::new(
            self.grid.into_par_iter().map(f).collect::<Vec<S>>(),
            self.resolution,
        )
    }

    pub fn operate_by_value_ref<F, S>(&self, f: F) -> Brane<S>
    where
        S: Send,
        F: Fn(&T) -> S,
    {
        Brane::new(self.grid.iter().map(f).collect::<Vec<S>>(), self.resolution)
    }

    pub fn downgrade(&self, factor: usize) -> Self
    where
        T: Copy + Sync + Send,
    {
        let res = self.resolution.release();
        Brane::create_by_index(Resolution::confine(res / factor), |jndex| {
            let kndex = jndex * factor;
            self.grid[kndex + (kndex / res) * res * (factor - 1)]
        })
    }
}

/* ## raws */

impl From<Brane<f64>> for Brane<u8> {
    fn from(brane: Brane<f64>) -> Self {
        brane.operate_by_value(|value| (value * 2.0_f64.powi(8) - 1.0) as u8)
    }
}

impl From<Brane<u8>> for Brane<f64> {
    fn from(brane: Brane<u8>) -> Self {
        brane.operate_by_value(|value| value as f64 / (2.0_f64.powi(8) - 1.0))
    }
}

impl Brane<u8> {
    /// save brane to a .tif file
    pub fn save_raw_low(&self, variable: String) {
        let path_name = format!("static/{}-u8-{}.tiff", variable, self.resolution.release());
        trace!("saving brane to {}", path_name);
        TiffEncoder::new(&mut fs::File::create(&Path::new(&path_name)).unwrap())
            .unwrap()
            .write_image::<colortype::Gray8>(
                self.resolution.release() as u32,
                self.resolution.release() as u32,
                &self.grid,
            )
            .unwrap();
    }

    /// load brane with a given name from a .tif file
    pub fn load_raw_low(variable: String) -> Self {
        let mut varextended = variable;
        varextended.push_str("-u8");

        let resolution: Resolution = {
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir("static") {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.starts_with(&varextended) {
                            files.push(name);
                        }
                    }
                }
            }
            let mut resolutions = files
                .iter()
                .map(|file| {
                    file.split_once('.')
                        .expect("file should have one extension")
                        .0
                        .rsplit_once('-')
                        .expect("last part should be resolution")
                        .1
                        .parse::<usize>()
                        .expect("variable contains something weird")
                })
                .collect::<Vec<usize>>();
            resolutions.sort_unstable();
            Resolution::confine(
                resolutions
                    .pop()
                    .expect("found no brane for specified variable"),
            )
        };

        let path_name = format!("static/{}-{}.tiff", varextended, resolution.release());
        trace!("loading brane from {}", path_name);
        let mut file = fs::File::open(&Path::new(&path_name)).unwrap();
        let mut tiff = Decoder::new(&mut file).unwrap();

        Self::new(
            match tiff.read_image().unwrap() {
                DecodingResult::U8(vector) => vector,
                _ => panic!(),
            },
            resolution,
        )
    }
}

impl From<Brane<f64>> for Brane<u16> {
    fn from(brane: Brane<f64>) -> Self {
        brane.operate_by_value(|value| (value * 2.0_f64.powi(16) - 1.0) as u16)
    }
}

impl From<Brane<u16>> for Brane<f64> {
    fn from(brane: Brane<u16>) -> Self {
        brane.operate_by_value(|value| value as f64 / (2.0_f64.powi(16) - 1.0))
    }
}

impl Brane<u16> {
    /// save brane to a .tif file
    pub fn save_raw(&self, variable: String) {
        let path_name = format!("static/{}-u16-{}.tiff", variable, self.resolution.release());
        trace!("saving brane to {}", path_name);
        TiffEncoder::new(&mut fs::File::create(&Path::new(&path_name)).unwrap())
            .unwrap()
            .write_image::<colortype::Gray16>(
                self.resolution.release() as u32,
                self.resolution.release() as u32,
                &self.grid,
            )
            .unwrap();
    }

    /// load brane with a given name from a .tif file
    pub fn load_raw(variable: String) -> Self {
        let mut varextended = variable;
        varextended.push_str("-u16");

        let resolution: Resolution = {
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir("static") {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.starts_with(&varextended) {
                            files.push(name);
                        }
                    }
                }
            }
            let mut resolutions = files
                .iter()
                .map(|file| {
                    file.split_once('.')
                        .expect("file should have one extension")
                        .0
                        .rsplit_once('-')
                        .expect("last part should be resolution")
                        .1
                        .parse::<usize>()
                        .expect("variable contains something weird")
                })
                .collect::<Vec<usize>>();
            resolutions.sort_unstable();
            Resolution::confine(
                resolutions
                    .pop()
                    .expect("found no brane for specified variable"),
            )
        };

        let path_name = format!("static/{}-{}.tiff", varextended, resolution.release());
        trace!("loading brane from {}", path_name);
        let mut file = fs::File::open(&Path::new(&path_name)).unwrap();
        let mut tiff = Decoder::new(&mut file).unwrap();

        Self::new(
            match tiff.read_image().unwrap() {
                DecodingResult::U16(vector) => vector,
                _ => panic!(),
            },
            resolution,
        )
    }
}

/* ## units */

impl<T> Brane<T>
where
    T: Clone + Copy + PartialOrd,
{
    fn quantile(&self, q: f64) -> T {
        let mut v = self.grid.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[(v.len() as f64 * q) as usize]
    }

    fn minimum(&self) -> T {
        *self
            .grid
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn maximum(&self) -> T {
        *self
            .grid
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn median(&self) -> T {
        self.quantile(0.5)
    }
}

impl Brane<f64> {
    /* # saving and loading */

    /// save brane to a .tif file
    pub fn save_f64(&self, variable: String) {
        Brane::<u16>::from(self.clone()).save_raw(variable);
    }

    /// load brane with a given name from a .tif file
    pub fn load_f64(variable: String) -> Self {
        Self::from(Brane::<u16>::load_raw(variable))
    }

    /* # statistics */

    fn mean(&self) -> f64 {
        self.grid.iter().sum::<f64>() / self.resolution.square() as f64
    }

    fn variance(&self) -> f64 {
        let mean = self.mean();
        self.grid.iter().map(|j| (j - mean).powi(2)).sum::<f64>()
            / (self.resolution.square() - 1) as f64
    }

    pub fn stats_raw(&self) {
        println!("statistics for brane");
        println!("    minimum:    {:.6}", self.minimum());
        println!("    mediam:     {:.6}", self.median());
        println!("    mean:       {:.6}", self.mean());
        println!("    maximum:    {:.6}", self.maximum());
        println!("    deviation:  {:.6}", self.variance().sqrt());
    }

    /* # utility */

    fn normalise_raw(&mut self) {
        let (min, max) = (self.minimum(), self.maximum());
        for value in self.grid.iter_mut() {
            *value = value
                .mul_add((max - min).recip(), -min * (max - min).recip())
                .min(1.)
                .max(0.);
        }
    }

    /// get a value interpolated from nearest coordinates
    fn compute(&self, datum: DatumRe) -> f64 {
        let target = datum * self.resolution.into();
        let corners = target.rhombus();
        let values = corners.map(|datum| self.grid[datum.unravel(self.resolution)]);
        let corners = corners.map(DatumRe::from);
        Spline::from_vec(vec![
            Key::new(
                corners[0].y,
                Spline::from_vec(vec![
                    Key::new(corners[0].x, values[0], Interpolation::Linear),
                    Key::new(corners[1].x, values[1], Interpolation::default()),
                ])
                .sample(target.x)
                .unwrap(),
                Interpolation::Linear,
            ),
            Key::new(
                corners[3].y,
                Spline::from_vec(vec![
                    Key::new(corners[2].x, values[2], Interpolation::Linear),
                    Key::new(corners[3].x, values[3], Interpolation::default()),
                ])
                .sample(target.x)
                .unwrap(),
                Interpolation::default(),
            ),
        ])
        .sample(target.y)
        .unwrap()
    }

    /// change the resolution of the brane
    pub fn upscale_raw(&self, target: Resolution) -> Self {
        match self.resolution == target {
            true => self.clone(),
            false => Self::create_by_index(target, |j| {
                self.compute(DatumZa::enravel(j, target).cast(target))
            }),
        }
    }
}

/* # impl unit branes */

impl<U> Brane<U>
where
    U: Unit + Send + Copy,
    U::Raw: Send,
{
    /// unwrap monad
    pub fn release(&self) -> Brane<U::Raw> {
        self.operate_by_value_ref(|value| value.release())
    }
}

impl<U> Brane<U>
where
    U: Unit + Send + Copy,
    U::Raw: Send,
    Brane<u8>: From<Brane<U::Raw>>,
    Brane<u16>: From<Brane<U::Raw>>,
{
    /// save brane to a .tif file
    pub fn save_low(&self, variable: String) {
        Brane::<u8>::from(self.release()).save_raw_low(variable);
    }

    /// save brane to a .tif file
    pub fn save(&self, variable: String) {
        Brane::<u16>::from(self.release()).save_raw(variable);
    }
}
impl<U> Brane<U>
where
    U: Unit + Send,
    U::Raw: Send,
    Brane<U::Raw>: From<Brane<u8>>,
    Brane<U::Raw>: From<Brane<u16>>,
{
    /// load brane with a given name from a .tif file
    pub fn load_low(variable: String) -> Self {
        Brane::<U::Raw>::from(Brane::<u8>::load_raw_low(variable))
            .operate_by_value(|value| U::confine(value))
    }

    /// load brane with a given name from a .tif file
    pub fn load(variable: String) -> Self {
        Brane::<U::Raw>::from(Brane::<u16>::load_raw(variable))
            .operate_by_value(|value| U::confine(value))
    }
}

impl<U> Brane<U>
where
    U: Unit<Raw = f64> + Send + Copy,
{
    pub fn stats(&self) {
        self.release().stats_raw();
    }

    /// scale values to the [0,1] interval
    pub fn normalise(&self) -> Self {
        let mut raw = self.release();
        raw.normalise_raw();
        raw.operate_by_value(|value| U::confine(value))
    }

    /// change the resolution of the brane
    pub fn upscale(&self, target: Resolution) -> Self {
        match self.resolution == target {
            true => self.clone(),
            false => {
                let raw = self.release();
                Self::create_by_index(target, |j| {
                    U::confine(raw.compute(DatumZa::enravel(j, target).cast(target)))
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::units::Elevation;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    /* # branes */

    #[test]
    fn brane_type_conversion() {
        let res = Resolution::confine(2);
        let brane_f64 = Brane {
            grid: vec![0.0_f64, 0.5, 1.0, 0.5],
            resolution: res,
        };
        let brane_u16 = Brane {
            grid: vec![0_u16, 32767, 65535, 32767],
            resolution: res,
        };
        assert_eq!(Brane::<u16>::from(brane_f64.clone()).grid, brane_u16.grid);
        assert_float_eq!(
            Brane::<f64>::from(brane_u16.clone()).grid,
            brane_f64.grid,
            rmax <= vec![4.0 * EPSILON; 4]
        );
    }

    #[test]
    fn brane_save_load() {
        let brane = Brane::<u16>::new(vec![0, 16384, 32768, 65535], Resolution::confine(2));
        brane.save_raw("test-write-rawu16".to_string());
        assert!(Path::new("static/test-write-rawu16-u16-2.tiff").exists());
        assert_eq!(
            Brane::<u16>::load_raw("test-write-rawu16".to_string()).grid,
            brane.grid
        );

        /*
        let brane = Brane::<f64>::new(vec![0.0, 0.25, 0.5, 0.75], Resolution::confine(2));
        brane.save_raw("test-write-rawf64".to_string());
        assert!(Path::new("static/test-write-rawf64-u16-2.tif").exists());
        assert_float_eq!(
            Brane::<f64>::load_raw("test-write-rawf64".to_string()).grid,
            brane.grid,
            rmax <= vec![EPSILON; 4]
        );
        */

        let brane = Brane::<Elevation>::new(
            vec![
                Elevation::confine(0.0),
                Elevation::confine(0.25),
                Elevation::confine(0.5),
                Elevation::confine(0.75),
            ],
            Resolution::confine(2),
        );
        brane.save("test-write-elevation".to_string());
        assert!(Path::new("static/test-write-elevation-u16-2.tiff").exists());
        assert_float_eq!(
            Brane::<Elevation>::load("test-write-elevation".to_string())
                .release()
                .grid,
            brane.release().grid,
            rmax <= vec![EPSILON; 4]
        );

        fs::remove_file("static/test-write-rawu16-u16-2.tiff").expect("test failed");
        //fs::remove_file("static/test-write-rawf64-u16-2.tif").expect("test failed");
        fs::remove_file("static/test-write-elevation-u16-2.tiff").expect("test failed");
    }

    #[test]
    fn brane_upscale() {
        let brane = Brane::new(
            vec![
                Elevation::confine(0.0),
                Elevation::confine(1.0),
                Elevation::confine(2.0),
                Elevation::confine(3.0),
            ],
            Resolution::confine(2),
        );
        let upscaled = brane.upscale(Resolution::confine(3));
        assert_eq!(upscaled.grid.len(), 9);
        assert_float_eq!(
            upscaled.release().grid,
            vec![
                0.0,
                2.0 / 3.0,
                2.0 / 3.0,
                4.0 / 3.0,
                2.0,
                2.0,
                4.0 / 3.0,
                2.0,
                2.0
            ],
            abs <= vec![EPSILON; 9]
        );
    }
}
