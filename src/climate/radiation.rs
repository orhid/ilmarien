use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::{Hexagon, HoneyCellPlanar},
    },
    vars::*,
};
use log::trace;
use rayon::prelude::*;

/* # insolation */

fn vector_elevation(datum: &DatumRe, elevation: f64) -> [f64; 3] {
    let cdatum = datum.centre();
    [cdatum.x, cdatum.y, elevation]
}

fn insolation_sol(datum: &DatumRe, sol: [f64; 3]) -> f64 {
    // turns out, the influence of both elevation and slope is negligable
    let pnt = vector_elevation(datum, 0.0);
    let solward = [sol[0] - pnt[0], sol[1] - pnt[1], sol[2] - pnt[2]];
    let solward_norm = solward.map(|j| j.powi(2)).iter().sum::<f64>().sqrt();
    solward_norm.powi(-2) * solward[2] * solward_norm.recip()
}

fn insolation_dt(datum: &DatumRe, solar_pos: f64) -> f64 {
    DatumZa { x: 0, y: 0 }
        .ball_planar(SOL_DETAIL)
        .into_iter()
        .map(|sol| {
            insolation_sol(
                datum,
                vector_elevation(
                    &(DatumRe::from(sol)
                        + DatumRe {
                            x: 0.0,
                            y: solar_pos,
                        }),
                    SOL_HEIGHT,
                ),
            )
        })
        .sum::<f64>()
}

/// calculate insolation – the amount of radiation reaching the surface over a single day
pub fn insolation(resolution: usize, solar_pos: f64) -> Brane<f64> {
    trace!("calculating insolation model");

    // should return numbers from the [0,1] interval
    // mean at around 0.3
    Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| insolation_dt(&DatumZa::enravel(j, resolution).cast(resolution), solar_pos))
            .collect::<Vec<f64>>(),
    )
    .mul_add(2.56, -5.57573)
}

/* # temperature */

fn temperature_update_ix(insolation: f64, temperature: f64, continentality: f64) -> f64 {
    // this converges to insolation ^ 1.44.recip() for continentality one
    // for exponent above two it becomes chaotic in temperature for large enough insolation
    //   as this is basically the logistic map
    // could also include albedo:
    //   continentality * (albedo * (insolation - temperature.powf(1.44)) + temperature) + ...
    continentality * (insolation - temperature.powf(1.44) + temperature)
        + (1.0 - continentality) * temperature
}

/// update temperature values based on insolation
pub fn temperature_update(
    insolation: &Brane<f64>,
    temperature: &mut Brane<f64>,
    continentality: &Brane<f64>,
) {
    trace!("updating temperature model");

    for j in 0..temperature.resolution.pow(2) {
        temperature.grid[j] = temperature_update_ix(
            insolation.grid[j],
            temperature.grid[j],
            continentality.grid[j],
        );
    }
}

/// calculate temperature lapse rate
pub fn lapse(altitude: f64) -> f64 {
    (altitude - INIT_OCEAN_LEVEL) * LAPSE_RATE
}

/* # pressure */

/* this can be useful later
#[allow(dead_code)]
fn pressure_elevation(pressure: f64, elevation: f64, temperature: f64) -> f64 {
    pressure * (LAPSE_CONST * elevation * temperature.recip()).exp()
}
*/

/// calculate pressure at ocean level
fn pressure(temperature: &Brane<f64>) -> Brane<f64> {
    trace!("calculating pressure at ocean level");
    temperature.clone().mul_add(-1.0, 1.0)
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
    fn insolation_values() {
        let brane = insolation(6, 0.0);
        assert!(*brane.grid.iter().ord_subset_max().unwrap() < 1.0);
        assert!(*brane.grid.iter().ord_subset_min().unwrap() > 0.0);
        assert_float_eq!(brane.grid.iter().sum::<f64>() / 36.0, 0.3, abs <= 0.001);

        assert_float_eq!(brane.grid[0], 0.900178, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.117151, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.286945, abs <= EPSILON);

        let brane = insolation(6, 0.1);
        assert_float_ne!(brane.grid[0], 0.900178, abs <= EPSILON);
        assert_float_ne!(brane.grid[8], 0.117151, abs <= EPSILON);
        assert_float_ne!(brane.grid[24], 0.286945, abs <= EPSILON);
    }

    #[test]
    fn temperature_values() {
        let mut brane = insolation(6, 0.0);
        temperature_update(&brane.clone(), &mut brane, &Brane::from(vec![1.0; 36]));
        assert_float_eq!(brane.grid[0], 0.940882, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.188699, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.408226, abs <= EPSILON);
    }

    #[test]
    fn pressure_values() {
        let brane = pressure(&Brane::from(
            (0..36).map(|j| j as f64 / 36.0).collect::<Vec<f64>>(),
        ));
        assert_float_eq!(brane.grid[0], 1.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.777777, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.333333, abs <= EPSILON);
    }
}
