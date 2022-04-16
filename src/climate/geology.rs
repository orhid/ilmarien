use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        honeycomb::HoneyCellToroidal,
    },
    climate::vegetation::Vege,
};
use log::trace;
use noise::{NoiseFn, OpenSimplex, Seedable};
use rayon::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::collections::VecDeque;
use std::f64::consts::TAU;

/* # bedrock generation */

const GEO_DETAIL: i32 = 12; // number of octaves in noise generation
const GEO_SCALE: f64 = 0.84; // scale for generated noise
const AMP_FACTOR: f64 = 1.44; // base for amplitude geometric series
const BLW_FACTOR: f64 = 1.68; // should blow results to [-1,1] range
const DST_FACTOR: f64 = 0.866025403784; // should slightly undistort terrain
pub const INIT_OCEAN_LEVEL: f64 = 0.25; // initial ocean level

fn elevation_curve() -> Spline<f64, f64> {
    Spline::from_vec(vec![
        Key::new(-1., 0., Interpolation::Linear),
        Key::new(-0.4744, 0.1843, Interpolation::Linear),
        Key::new(-0.2806, 0.2157, Interpolation::Linear),
        Key::new(-0.1693, 0.2392, Interpolation::Linear),
        Key::new(-0.1094, 0.2471, Interpolation::Linear),
        Key::new(-0.0416, 0.2549, Interpolation::Linear),
        Key::new(0.0367, 0.2627, Interpolation::Linear),
        Key::new(0.1326, 0.2784, Interpolation::Linear),
        Key::new(0.2353, 0.3098, Interpolation::Linear),
        Key::new(0.3628, 0.3725, Interpolation::Linear),
        Key::new(1., 1., Interpolation::Linear),
    ])
}

fn bedrock_level_dt(datum: &DatumRe, noise: &OpenSimplex, curve: &Spline<f64, f64>) -> f64 {
    let x: f64 = TAU * datum.x;
    let y: f64 = TAU * datum.y;
    curve
        .clamped_sample(
            BLW_FACTOR
                * (0..GEO_DETAIL)
                    .map(|level| {
                        let freq = GEO_SCALE * 2.0f64.powi(level);
                        AMP_FACTOR.powi(-level)
                            * noise.get([
                                freq * x.cos(),
                                freq * x.sin(),
                                freq * DST_FACTOR * y.cos(),
                                freq * DST_FACTOR * y.sin(),
                            ])
                    })
                    .sum::<f64>()
                * (1.0 - AMP_FACTOR.recip()),
        )
        .unwrap()
}

/**
 * generate a bedrock elevation model from Perlin noise
 * the values will range from 0.0 to 1.0
 * they correspond to the difference between 0 and 13824 meters
 */
pub fn bedrock_level(resolution: usize, seed: u32) -> Brane<f64> {
    trace!("generating bedrock altitude model");
    Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                bedrock_level_dt(
                    &DatumZa::enravel(j, resolution).cast(resolution),
                    &OpenSimplex::new().set_seed(seed),
                    &elevation_curve(),
                )
            })
            .collect::<Vec<f64>>(),
    )
}

pub fn bedrock_vege(altitude: &Brane<f64>) -> Brane<Option<Vege>> {
    Brane::from(
        (0..altitude.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                if altitude.grid[j] < INIT_OCEAN_LEVEL {
                    None
                } else {
                    Some(Vege::Stone)
                }
            })
            .collect::<Vec<Option<Vege>>>(),
    )
}

/* # continentality */

const LEVEL_MOUNTAIN: f64 = 0.12;

pub fn altitude_with_ocean(altitude: &Brane<f64>, ocean: f64) -> Brane<f64> {
    Brane::from(
        (0..altitude.resolution.pow(2))
            .into_par_iter()
            .map(|j| altitude.grid[j].max(ocean))
            .collect::<Vec<f64>>(),
    )
}

/// find mountain tiles
fn mountain_tiles(altitude: &Brane<f64>, ocean: f64) -> Brane<bool> {
    Brane::from(
        (0..altitude.resolution.pow(2))
            .into_par_iter()
            .map(|j| altitude.grid[j] > ocean + LEVEL_MOUNTAIN)
            .collect::<Vec<bool>>(),
    )
}

/// find ocean tiles
pub fn ocean_tiles(altitude: &Brane<f64>, ocean: f64) -> Brane<bool> {
    Brane::from(
        (0..altitude.resolution.pow(2))
            .into_par_iter()
            .map(|j| altitude.grid[j] < ocean)
            .collect::<Vec<bool>>(),
    )
}

/// find distance to closest ocean, going around mountains
pub fn continentality(altitude: &Brane<f64>, ocean: f64) -> Brane<f64> {
    trace!("calculating continentality coefficients");
    let step = 12.0 / altitude.resolution as f64;
    let mut continentality = Brane::from(
        mountain_tiles(altitude, ocean)
            .grid
            .into_par_iter()
            .map(|b| if b { 1.0 } else { f64::NAN })
            .collect::<Vec<f64>>(),
    );

    // preopoulate oceans with zeros
    let mut ocean_datums = VecDeque::new();
    let ocean_tiles = ocean_tiles(altitude, ocean);
    for index in 0..altitude.resolution.pow(2) {
        if ocean_tiles.grid[index] {
            continentality.grid[index] = 0.0;
            ocean_datums.push_back(DatumZa::enravel(index, altitude.resolution));
        }
    }

    // flood fill from ocean datums
    while !ocean_datums.is_empty() {
        let here = ocean_datums.pop_front().unwrap();
        for datum in here.ambit_toroidal(altitude.resolution as i32) {
            let index = datum.unravel(altitude.resolution);
            if continentality.grid[index].is_nan() {
                continentality.grid[index] =
                    continentality.grid[here.unravel(altitude.resolution)] + step;
                ocean_datums.push_back(datum);
            }
        }
    }

    // check if any tiles were left stranded inside mountain regions
    for j in 0..altitude.resolution.pow(2) {
        if continentality.grid[j].is_nan() || continentality.grid[j] > 1.0 {
            continentality.grid[j] = 1.0 // 1.0
        }
    }
    continentality
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn bedrock_level_values() {
        let brane = bedrock_level(6, 0);
        assert_float_eq!(brane.grid[0], 0.262396, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.295638, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.247292, abs <= EPSILON);

        let brane = bedrock_level(6, 1);
        assert_float_ne!(brane.grid[0], 0.262396, abs <= EPSILON);
        assert_float_ne!(brane.grid[8], 0.295638, abs <= EPSILON);
        assert_float_ne!(brane.grid[24], 0.247292, abs <= EPSILON);
    }

    #[test]
    fn bedrock_level_tileability() {
        let noise = OpenSimplex::new();
        let curve = elevation_curve();
        assert_float_eq!(
            bedrock_level_dt(&DatumRe::new(0.0, 0.1), &noise, &curve),
            bedrock_level_dt(&DatumRe::new(0.0, 1.1), &noise, &curve),
            abs <= EPSILON,
        );
        assert_float_eq!(
            bedrock_level_dt(&DatumRe::new(0.1, 0.0), &noise, &curve),
            bedrock_level_dt(&DatumRe::new(1.1, 0.0), &noise, &curve),
            abs <= EPSILON,
        );
    }

    #[test]
    fn continentality_values() {
        let brane = continentality(
            &Brane::from((0..36).map(|j| j as f64 / 36.0).collect::<Vec<f64>>()),
            0.25,
        );
        assert_float_eq!(brane.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[12], 1.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 1.0, abs <= EPSILON);
    }
}
