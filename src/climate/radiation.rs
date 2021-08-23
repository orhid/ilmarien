use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa, Resolution},
        honeycomb::{Hexagon, HoneyCellPlanar, HoneyCellToroidal},
    },
    util::constants::*,
};
use log::info;
use nalgebra::Vector3;
use ordered_float::OrderedFloat;
use petgraph::graph::{Graph, NodeIndex};
use rayon::prelude::*;
use std::collections::HashMap;

/* # insolation */

fn vector_elevation(datum: &DatumRe, elevation: f64) -> Vector3<f64> {
    let cdatum = datum.centre();
    Vector3::new(cdatum.x, cdatum.y, elevation)
}

fn insolation_sol(datum: &DatumRe, sol: &Vector3<f64>) -> f64 {
    // turns out, the influence of both elevation and slope is negligable
    let solward = sol - vector_elevation(datum, 0.0);
    let solward_norm = solward.norm();
    solward_norm.powi(-2) * solward.dot(&Vector3::new(0.0, 0.0, 1.0)) * solward_norm.recip()
}

fn insolation_dt(datum: &DatumRe, solar_pos: f64) -> f64 {
    DatumZa::from(*datum)
        .ball_planar(SOL_DETAIL)
        .into_iter()
        .map(|sol| insolation_sol(datum, &vector_elevation(&DatumRe::from(sol), solar_pos)))
        .sum::<f64>()
}

/// calculate insolation – the amount of radiation reaching the surface over a single day
pub fn insolation(resolution: Resolution, solar_pos: f64) -> Brane<f64> {
    info!("calculating insolation model");

    let mut brane = Brane::from(
        Brane::<f64>::par_iter(resolution)
            .map(|datum| insolation_dt(&datum, solar_pos))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "insolation".to_string();
    brane
}

/*
/* # temperature */

/// initialise temperature to a given value in degrees Kelvin
fn temperature_initialise(resolution: Resolution, insolation: &Brane<f64>) -> Brane<f64> {
    info!("initialising temperature");
    let mut brane = Brane::from(
        Brane::<f64>::par_iter(resolution)
            .map(|datum| insolation.get(&datum).mul_add(SOL_POWER, INIT_TEMP))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "temperature".to_string();
    brane
}

/// calculate temperature diffusion
fn temperature_diffuse(temperature: &mut Brane<f64>, surface: &Brane<u8>) {
    info!("calculating temperature diffusion");

    for j in 0..temperature.resolution * 12 {
        temperature.grid = temperature
            .into_par_iter()
            .map(|datum| {
                diffusion_medium(
                    &datum,
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
pub fn temperature(
    resolution: Resolution,
    insolation: &Brane<f64>,
    surface: &Brane<u8>,
) -> Brane<f64> {
    let mut temperature = temperature_initialise(resolution, insolation);
    temperature_diffuse(&mut temperature, surface);
    temperature
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
pub fn pressure_calculate(resolution: Resolution, temperature: &Brane<f64>) -> Brane<f64> {
    info!("calculating pressure at ocean level");
    let mut brane = Brane::from(
        Brane::<f64>::par_iter(resolution)
            .map(|datum| {
                temperature
                    .get(&datum)
                    .recip()
                    .mul_add(GAS_CONST, INIT_PRES)
            })
            .collect::<Vec<f64>>(),
    );
    brane.variable = "pressure".to_string();
    brane
}

/// calculate pressure gradient for moisture transportation, including elevation changes
pub fn pressure_gradient(pressure: &Brane<f64>) -> (Graph<Coordinate<i32>, f64>, Vec<NodeIndex>) {
    info!("calculating pressure gradient");

    let mut graph = Graph::<Coordinate<i32>, f64>::new();
    let mut nodes = HashMap::<Coordinate<i32>, NodeIndex>::new();
    let mut roots = Vec::<NodeIndex>::new();
    for datum in pressure.exact_iter() {
        let here = graph.add_node(datum);
        nodes.insert(datum, here);
    }
    for datum in pressure.exact_iter() {
        let minbr = *pressure
            .exact_ambit(&datum)
            .iter()
            .min_by_key(|nbr| OrderedFloat(pressure.read(&nbr)))
            .unwrap();
        let dif = pressure.read(&datum) - pressure.read(&minbr);
        if dif > 0.0 {
            graph.add_edge(nodes[&datum], nodes[&minbr], dif.recip().log10() * 0.1);
        } else {
            roots.push(nodes[&datum]);
        }
    }
    (graph, roots)
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn insolation_values() {
        let brane = insolation(6.into(), 1.0);
        assert_float_eq!(brane.grid[0], 2.224531, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.913083, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 1.982061, abs <= EPSILON);

        let brane = insolation(6.into(), 1.2);
        assert_float_ne!(brane.grid[0], 2.224531, abs <= EPSILON);
        assert_float_ne!(brane.grid[8], 1.913083, abs <= EPSILON);
        assert_float_ne!(brane.grid[24], 1.982061, abs <= EPSILON);
    }

    // test temperature
}
