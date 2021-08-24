use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
    },
    climate::{cosmos::Fabric, radiation::pressure_gradient},
    util::{constants::*, diffusion::diffuse_level},
};
use log::info;
use ordered_float::OrderedFloat as Orf;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;
use std::cmp::{max, min};

/* # rainfall */

/* ## evaporation */

fn evaporation_rate(temperature: f64, pressure: f64) -> f64 {
    (temperature * pressure.recip()).mul_add(108f64.recip(), -2.2)
}

fn evaporation_dt(
    datum: &DatumRe,
    surface: &Brane<Fabric>,
    temperature: &Brane<f64>,
    pressure: &Brane<f64>,
) -> f64 {
    match surface.get(&datum) {
        Fabric::Water => evaporation_rate(temperature.get(&datum), pressure.get(&datum)),
        Fabric::Ice | Fabric::Snow => {
            0.24 * evaporation_rate(temperature.get(&datum), pressure.get(&datum))
        }
        _ => 0.0,
    }
}

/// calculate evaporation rate
pub fn evaporation(
    resolution: usize,
    surface: &Brane<Fabric>,
    temperature: &Brane<f64>,
    pressure: &Brane<f64>,
) -> Brane<f64> {
    info!("calculating evaporation rate");

    let mut brane = Brane::from(
        Brane::<f64>::par_iter(resolution)
            .map(|datum| evaporation_dt(&datum, &surface, &temperature, &pressure))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "evaporation".to_string();
    brane
}

/* ## rainfall */

fn rainfall_node(
    target_level: f64,
    node: NodeIndex,
    gradient: &Graph<DatumZa, f64>,
    evaporation: &Brane<f64>,
    elevation: &Brane<f64>,
    rainfall: &mut Brane<f64>,
) -> f64 {
    let datum = &gradient[node];
    let level = elevation.get(&evaporation.cast(&datum));
    let moisture = evaporation.read(&datum)
        + gradient
            .edges_directed(node, Direction::Incoming)
            .map(|edge| {
                rainfall_node(
                    level,
                    edge.source(),
                    gradient,
                    evaporation,
                    elevation,
                    rainfall,
                )
            })
            .sum::<f64>();
    let frac = moisture
        * f64::from(min(
            max(Orf(0.054), Orf(target_level - level) * 8.0),
            Orf(1.0),
        ));
    rainfall.insert(&datum, frac);
    moisture - frac
}

/// calculate the amount of rainfall reaching the surface
pub fn rainfall(
    pressure: &Brane<f64>,
    evaporation: &Brane<f64>,
    elevation: &Brane<f64>,
) -> Brane<f64> {
    info!("calculating rainfall");

    let (gradient, roots) = pressure_gradient(pressure);
    let mut rainfall = Brane::<f64>::zeros(evaporation.resolution);
    for node in roots {
        rainfall_node(0.0, node, &gradient, evaporation, elevation, &mut rainfall);
    }

    for _ in 0..rainfall.resolution / 12 {
        rainfall.grid = rainfall
            .par_iter()
            .map(|datum| diffuse_level(&datum, &rainfall, &elevation))
            .collect::<Vec<f64>>();
    }
    rainfall.variable = "rainfall".to_string();
    rainfall
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn evaporation_values() {
        let surface = Brane::from((0..36).map(|_| Fabric::Water).collect::<Vec<Fabric>>());
        let temperature = Brane::from((0..36).map(|j| j as f64 + 273.0).collect::<Vec<f64>>());
        let pressure = Brane::from((0..36).map(|_| 1f64).collect::<Vec<f64>>());
        let brane = evaporation(6, &surface, &temperature, &pressure);
        assert_float_eq!(brane.grid[0], 0.327777, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.401851, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.549999, abs <= EPSILON);
    }

    #[test]
    fn rainfall_values() {
        let pressure = Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>());
        let evaporation = Brane::from((0..36).map(|j| (j % 3) as f64).collect::<Vec<f64>>());
        let elevation = Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>());
        let brane = rainfall(&pressure, &evaporation, &elevation);
        assert_float_eq!(brane.grid[0], 1.608919, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.398250, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.0, abs <= EPSILON);
    }
}
