use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::{Hexagon, HoneyCellPlanar},
    },
    climate::geology::INIT_OCEAN_LEVEL,
};
use log::trace;
use rayon::prelude::*;
use std::f64::consts::TAU;

const SOL_DETAIL: i32 = 12; // radius of suns taken for insolation calculation
const SOL_HEIGHT: f64 = 1.0; // altitue of the sun
const SOL_XDEV: f64 = 0.08; // amplidute of solar deviation
const SOL_YDEV: f64 = 0.12; // amplidute of solar deviation

/* # insolation */

fn vector_elevation(datum: &DatumRe, elevation: f64) -> [f64; 3] {
    let cdatum = datum.centre();
    [cdatum.x, cdatum.y, elevation]
}

fn insolation_sol(pnt: [f64; 3], sol: [f64; 3]) -> f64 {
    // turns out, the influence of both elevation and slope is negligable
    let solward = [sol[0] - pnt[0], sol[1] - pnt[1], sol[2] - pnt[2]];
    let solward_norm = solward.map(|j| j.powi(2)).iter().sum::<f64>().sqrt();
    solward_norm.powi(-2) * solward[2] * solward_norm.recip()
}

fn insolation_dt(datum: &DatumRe, solar_pos: f64) -> f64 {
    // should return numbers from the [0,1] interval
    // mean at around 0.3

    DatumZa { x: 0, y: 0 }
        .ball_planar(SOL_DETAIL)
        .into_par_iter()
        .map(|sol| {
            insolation_sol(
                vector_elevation(datum, 0.0),
                vector_elevation(
                    &(DatumRe::from(sol)
                        + DatumRe {
                            x: SOL_XDEV * (TAU * solar_pos).cos(),
                            y: SOL_YDEV * (TAU * solar_pos).sin(),
                        }),
                    SOL_HEIGHT,
                ),
            )
        })
        .sum::<f64>()
        .mul_add(2.56, -5.57573)
}

/// coldest possible temperature at zero continentality
const TMP_MINSTABLE: f64 = 0.18;

fn temperature_oceanlv_dt(month: f64, datum: &DatumRe, continentality: f64) -> f64 {
    let insolation = insolation_dt(datum, month); // * albedo
    insolation + TMP_MINSTABLE * insolation.mul_add(-3.0, 1.0) * continentality.mul_add(-1.0, 1.0)
}

/// temperature values at ocean level
pub fn temperature_oceanlv(month: f64, continentality: &Brane<f64>) -> Brane<f64> {
    trace!("calculating temperature model");

    // should return numbers from the [0,1] interval
    Brane::from(
        (0..continentality.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                temperature_oceanlv_dt(
                    month,
                    &DatumZa::enravel(j, continentality.resolution).cast(continentality.resolution),
                    continentality.grid[j],
                )
            })
            .collect::<Vec<f64>>(),
    )
}

const RATE_LAPSE: f64 = 0.72;

/// calculate temperature lapse rate
fn lapse_ix(temperature: f64, altitude: f64) -> f64 {
    temperature
        * ((1.0 - altitude + INIT_OCEAN_LEVEL) + (altitude - INIT_OCEAN_LEVEL) * (1.0 - RATE_LAPSE))
}

pub fn temperature(temperature_oceanlv: &Brane<f64>, altitude: &Brane<f64>) -> Brane<f64> {
    Brane::from(
        (0..temperature_oceanlv.resolution.pow(2))
            .into_par_iter()
            .map(|j| lapse_ix(temperature_oceanlv.grid[j], altitude.grid[j]))
            .collect::<Vec<f64>>(),
    )
}

/// calculate pressure at ocean level
fn pressure(temperature: &Brane<f64>) -> Brane<f64> {
    trace!("calculating pressure at ocean level");
    let smoothing_coef: f64 = 6.0_f64.recip();
    temperature
        .upscale((temperature.resolution as f64 * smoothing_coef) as usize)
        .upscale(temperature.resolution)
        .mul_add(-1.0, 1.0)
}

/// calculate pressure gradient
pub fn wind(temperature: &Brane<f64>) -> Flux<f64> {
    trace!("calculating pressure gradient");
    Flux::<f64>::from(&pressure(temperature))
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    use ord_subset::OrdSubsetIterExt;
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn temperature_oceanlv_values() {
        let temp_ocean = temperature_oceanlv(0.0, &Brane::from(vec![0.0; 36]));
        assert!(*temp_ocean.grid.iter().ord_subset_max().unwrap() < 1.0 - TMP_MINSTABLE);
        assert!(*temp_ocean.grid.iter().ord_subset_min().unwrap() > TMP_MINSTABLE);

        assert_float_eq!(temp_ocean.grid[0], 0.568309, abs <= EPSILON);
        assert_float_eq!(temp_ocean.grid[8], 0.270136, abs <= EPSILON);
        assert_float_eq!(temp_ocean.grid[24], 0.254137, abs <= EPSILON);

        let temp_land = temperature_oceanlv(0.0, &Brane::from(vec![1.0; 36]));
        assert!(*temp_land.grid.iter().ord_subset_max().unwrap() < 1.0);
        assert!(*temp_land.grid.iter().ord_subset_min().unwrap() > 0.0);
        assert_float_eq!(temp_land.grid.iter().sum::<f64>() / 36.0, 0.3, abs <= 0.001);

        assert_float_eq!(temp_land.grid[0], 0.844150, abs <= EPSILON);
        assert_float_eq!(temp_land.grid[8], 0.195948, abs <= EPSILON);
        assert_float_eq!(temp_land.grid[24], 0.161167, abs <= EPSILON);

        let temp_land = temperature_oceanlv(0.1, &Brane::from(vec![1.0; 36]));
        assert_float_ne!(temp_land.grid[0], 0.844150, abs <= EPSILON);
        assert_float_ne!(temp_land.grid[8], 0.195948, abs <= EPSILON);
        assert_float_ne!(temp_land.grid[24], 0.161167, abs <= EPSILON);
    }

    #[test]
    fn pressure_values() {
        let brane = pressure(&Brane::from(
            (0..36).map(|j| j as f64 / 36.0).collect::<Vec<f64>>(),
        ));
        assert_float_eq!(brane.grid[0], 1.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 1.0, abs <= EPSILON);
    }
}
