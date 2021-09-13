use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::{Hexagon, HoneyCellPlanar},
    },
    climate::cosmos::Fabric,
    util::diffusion::{diffuse_medium, Medium},
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
    DatumZa::from(*datum)
        .ball_planar(SOL_DETAIL)
        .into_iter()
        .map(|sol| insolation_sol(datum, vector_elevation(&DatumRe::from(sol), solar_pos)))
        .sum::<f64>()
}

/// calculate insolation – the amount of radiation reaching the surface over a single day
pub fn insolation(resolution: usize, solar_pos: f64) -> Brane<f64> {
    trace!("calculating insolation model");

    let mut brane = Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| insolation_dt(&DatumZa::enravel(j, resolution).cast(resolution), solar_pos))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "insolation".to_string();
    brane
}

/* # temperature */

/// initialise temperature to a given value in degrees Kelvin
fn temperature_initialise(insolation: &Brane<f64>) -> Brane<f64> {
    trace!("initialising temperature");
    // this should be doable without cloning, need to work on the implementation
    insolation.clone().mul_add(SOL_POWER, INIT_TEMP)
    /*
    let mut brane = Brane::from(
        insolation
            .par_iter()
            .map(|datum| insolation.get(&datum).mul_add(SOL_POWER, INIT_TEMP))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "temperature".to_string();
    brane
        */
}

/// calculate temperature diffusion
fn temperature_diffuse(surface: &Brane<Fabric>, temperature: &mut Brane<f64>) {
    trace!("calculating temperature diffusion");

    for j in 0..temperature.resolution * 12 {
        temperature.grid = (0..temperature.resolution.pow(2))
            .into_par_iter()
            .map(|k| {
                diffuse_medium(
                    &DatumZa::enravel(k, temperature.resolution).cast(temperature.resolution),
                    match j % 6 {
                        0 => Medium::Air,
                        _ => Medium::Ocean,
                    },
                    temperature,
                    surface,
                )
            })
            .collect::<Vec<f64>>();
    }
}

/// calculate average temperature
pub fn temperature(insolation: &Brane<f64>, surface: &Brane<Fabric>) -> Brane<f64> {
    let mut temperature = temperature_initialise(insolation);
    temperature_diffuse(surface, &mut temperature);
    temperature
}

/// calculate temperature lapse rate
pub fn lapse(elevation: f64) -> f64 {
    (elevation - INIT_OCEAN_LEVEL) * LAPSE_RATE
}

/* # pressure */

/*
// this can be useful later

#[allow(dead_code)]
fn pressure_elevation(pressure: f64, elevation: f64, temperature: f64) -> f64 {
    pressure * (LAPSE_CONST * elevation * temperature.recip()).exp()
}
*/

/// calculate pressure at ocean level
pub fn pressure(temperature: &Brane<f64>) -> Brane<f64> {
    trace!("calculating pressure at ocean level");
    let mut brane = Brane::from(
        (0..temperature.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                temperature
                    .get(&DatumZa::enravel(j, temperature.resolution).cast(temperature.resolution))
                    .recip()
                    .mul_add(GAS_CONST, INIT_PRES)
            })
            .collect::<Vec<f64>>(),
    );
    brane.variable = "pressure".to_string();
    brane
}

/// calculate pressure gradient
pub fn wind(pressure: &Brane<f64>) -> Flux<f64> {
    trace!("calculating pressure gradient");

    Flux::<f64>::from(pressure)
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn insolation_values() {
        let brane = insolation(6, 1.0);
        assert_float_eq!(brane.grid[0], 2.224531, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.913083, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 1.982061, abs <= EPSILON);

        let brane = insolation(6, 1.2);
        assert_float_ne!(brane.grid[0], 2.224531, abs <= EPSILON);
        assert_float_ne!(brane.grid[8], 1.913083, abs <= EPSILON);
        assert_float_ne!(brane.grid[24], 1.982061, abs <= EPSILON);
    }

    #[test]
    fn temperature_values() {
        let brane = temperature(
            &Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>()),
            &Brane::from((0..36).map(|_| Fabric::Stone).collect::<Vec<Fabric>>()),
        );
        assert_float_eq!(brane.grid[0], 5294.517604, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 5279.538180, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 5339.466849, abs <= EPSILON);
    }

    #[test]
    fn pressure_values() {
        let brane = pressure(&Brane::from(
            (0..36).map(|j| j as f64 + 273.0).collect::<Vec<f64>>(),
        ));
        assert_float_eq!(brane.grid[0], 1.027472, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.012455, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.984848, abs <= EPSILON);
    }
}
