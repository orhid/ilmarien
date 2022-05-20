use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::HoneyCellToroidal,
    },
    climate::{hydrology::shed, vegetation::Vege},
};
use log::trace;
use noise::{NoiseFn, OpenSimplex, Seedable};
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::collections::VecDeque;
use std::f64::consts::TAU;

const SQRT3B2: f64 = 0.8660254;

/* # bedrock generation */

const GEO_DETAIL: i32 = 12; // number of octaves in noise generation
const GEO_SCALE: f64 = 0.5; //0.375; // scale for generated noise
const AMP_FACTOR: f64 = 1.5652; // base for amplitude geometric series
const BLW_FACTOR: f64 = 1.6725; // should blow results to [-1,1] range

fn elevation_curve() -> Spline<f64, f64> {
    // curve moves the mode of the distribution
    let step = 256f64.recip();
    let shelf: f64 = 0.24;
    Spline::from_vec(vec![
        Key::new(0., 0., Interpolation::Linear),
        Key::new(0.01, shelf - 4.0 * step, Interpolation::Linear),
        Key::new(0.06, shelf, Interpolation::Linear),
        Key::new(0.24, shelf + 8.0 * step, Interpolation::Linear),
        Key::new(1., 1., Interpolation::Linear),
    ])
}

fn fractional_brownian_motion(x: f64, y: f64, noise: &OpenSimplex) -> f64 {
    (0..GEO_DETAIL)
        .map(|level| {
            let freq = GEO_SCALE * 2.0f64.powi(level);
            AMP_FACTOR.powi(-level)
                * noise.get([
                    freq * x.cos(),
                    freq * x.sin(),
                    freq * SQRT3B2 * y.cos(),
                    freq * SQRT3B2 * y.sin(),
                ])
        })
        .sum::<f64>()
}

fn toroidal_noise_dt(datum: &DatumRe, noise: &OpenSimplex) -> f64 {
    let x: f64 = TAU * datum.x;
    let y: f64 = TAU * (datum.x + datum.y);
    let warp = fractional_brownian_motion(
        x + fractional_brownian_motion(x, y, noise),
        y + fractional_brownian_motion(x, y, noise),
        noise,
    );
    warp * BLW_FACTOR * (1. - AMP_FACTOR.recip())
}

fn bedrock_level_dt(datum: &DatumRe, noises: &[OpenSimplex], curve: &Spline<f64, f64>) -> f64 {
    curve
        .clamped_sample(
            0.72 * noises
                .iter()
                .map(|noise| toroidal_noise_dt(datum, noise).powi(2))
                .sum::<f64>(),
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
    let noises = (0..4) // three noises give chi sq distribution
        .map(|j| OpenSimplex::new().set_seed(seed + j))
        .collect::<Vec<OpenSimplex>>();
    Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                bedrock_level_dt(
                    &DatumZa::enravel(j, resolution).cast(resolution),
                    &noises,
                    &elevation_curve(),
                )
            })
            .collect::<Vec<f64>>(),
    )
}

pub fn bedrock_vege(altitude: &Brane<f64>, ocean: f64) -> Brane<Option<Vege>> {
    Brane::from(
        (0..altitude.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                if altitude.grid[j] < ocean {
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

/* # erosion */

const MAX_RAIN: f64 = 24.0;
const MAX_DELTA: f64 = 0.1296;
const BOUNCEBACK: f64 = 0.012;

fn erode_nd(
    node: NodeIndex,
    altitude: &mut Brane<f64>,
    shed: &Brane<f64>,
    gradient: &Graph<DatumZa, f64>,
) {
    let alt_here = altitude.read(&gradient[node]);
    for edge in gradient.edges_directed(node, Direction::Incoming) {
        let source = edge.source();
        let child = gradient[source];
        let rain = shed.read(&child);
        if rain > 1.0 {
            altitude.grid[child.unravel(altitude.resolution)] = alt_here
                + MAX_DELTA.min((altitude.read(&child) - alt_here) * (MAX_RAIN.min(rain)).recip())
                + BOUNCEBACK;
        } else {
            // thermal erosion, could for example tend towards the average of its neighbours if it is
            // above sea level
        }
        erode_nd(source, altitude, shed, gradient);
    }
}

const EROSION_LOOP: usize = 1;

pub fn erode(altitude: &mut Brane<f64>, rain: &Brane<f64>) {
    trace!("calculating erosion");
    for _ in 0..EROSION_LOOP {
        let slope = Flux::<f64>::from(&altitude.clone());
        let shed = shed(&slope, rain);
        for node in &slope.roots {
            erode_nd(*node, altitude, &shed, &slope.graph);
            altitude.grid[slope.graph[*node].unravel(altitude.resolution)] += BOUNCEBACK;
        }
    }
}

pub fn ocean(altitude: &Brane<f64>) -> f64 {
    altitude.mde() - 256_f64.recip()
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn bedrock_level_values() {
        let brane = bedrock_level(6, 0);
        assert_float_eq!(brane.grid[0], 0.232781, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.163223, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.247007, abs <= EPSILON);

        let brane = bedrock_level(6, 1);
        assert_float_ne!(brane.grid[0], 0.232781, abs <= EPSILON);
        assert_float_ne!(brane.grid[8], 0.163223, abs <= EPSILON);
        assert_float_ne!(brane.grid[24], 0.247007, abs <= EPSILON);
    }

    #[test]
    fn bedrock_level_tileability() {
        let noise = OpenSimplex::new();
        let curve = elevation_curve();
        assert_float_eq!(
            bedrock_level_dt(&DatumRe::new(0.0, 0.1), &[noise], &curve),
            bedrock_level_dt(&DatumRe::new(0.0, 1.1), &[noise], &curve),
            abs <= EPSILON,
        );
        assert_float_eq!(
            bedrock_level_dt(&DatumRe::new(0.1, 0.0), &[noise], &curve),
            bedrock_level_dt(&DatumRe::new(1.1, 0.0), &[noise], &curve),
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
