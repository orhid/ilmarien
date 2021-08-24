use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
    },
    climate::cosmos::Fabric,
    util::diffusion::diffuse_level,
};
use log::{info, warn};
use ord_subset::OrdSubsetIterExt;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;

/* # rainfall */

/* ## evaporation */

fn evaporation_rate(pressure: f64, temperature: f64) -> f64 {
    (temperature * pressure.recip()).mul_add(108f64.recip(), -2.2)
}

fn evaporation_dt(
    datum: &DatumRe,
    pressure: &Brane<f64>,
    surface: &Brane<Fabric>,
    temperature: &Brane<f64>,
) -> f64 {
    match surface.get(&datum) {
        Fabric::Water => evaporation_rate(pressure.get(&datum), temperature.get(&datum)),
        Fabric::Ice | Fabric::Snow => {
            0.24 * evaporation_rate(pressure.get(&datum), temperature.get(&datum))
        }
        _ => 0.0,
    }
}

/// calculate evaporation rate
pub fn evaporation(
    pressure: &Brane<f64>,
    surface: &Brane<Fabric>,
    temperature: &Brane<f64>,
) -> Brane<f64> {
    info!("calculating evaporation rate");

    if temperature.resolution != pressure.resolution {
        warn!("evaporation: branes at incompatible resolutions: temperature, pressure");
    }

    let mut brane = Brane::from(
        temperature
            .par_iter()
            .map(|datum| evaporation_dt(&datum, &pressure, &surface, &temperature))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "evaporation".to_string();
    brane
}

/* ## rainfall */

fn rainfall_node(
    target_level: f64,
    node: NodeIndex,
    elevation: &Brane<f64>,
    evaporation: &Brane<f64>,
    gradient: &Graph<DatumZa, f64>,
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
                    elevation,
                    evaporation,
                    gradient,
                    rainfall,
                )
            })
            .sum::<f64>();
    let frac = moisture
        * *[
            1.0,
            *[(target_level - level) * 8.0, 0.054]
                .iter()
                .ord_subset_max()
                .unwrap(),
        ]
        .iter()
        .ord_subset_min()
        .unwrap();
    rainfall.insert(&datum, frac);
    moisture - frac
}

/// calculate the amount of rainfall reaching the surface
pub fn rainfall(
    elevation: &Brane<f64>,
    evaporation: &Brane<f64>,
    pressure_flux: &Flux<f64>,
) -> Brane<f64> {
    info!("calculating rainfall");

    if evaporation.resolution != pressure_flux.resolution {
        warn!("rainfall: branes at incompatible resolutions: evaporation, pressure");
    }

    let mut rainfall = Brane::<f64>::zeros(evaporation.resolution);
    for node in &pressure_flux.roots {
        rainfall_node(
            0.0,
            *node,
            elevation,
            evaporation,
            &pressure_flux.graph,
            &mut rainfall,
        );
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
        let brane = evaporation(
            &Brane::from((0..36).map(|_| 1f64).collect::<Vec<f64>>()),
            &Brane::from((0..36).map(|_| Fabric::Water).collect::<Vec<Fabric>>()),
            &Brane::from((0..36).map(|j| j as f64 + 273.0).collect::<Vec<f64>>()),
        );
        assert_float_eq!(brane.grid[0], 0.327777, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.401851, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.549999, abs <= EPSILON);
    }

    #[test]
    fn rainfall_values() {
        let brane = rainfall(
            &Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>()),
            &Brane::from((0..36).map(|j| (j % 3) as f64).collect::<Vec<f64>>()),
            &Flux::<f64>::from(&Brane::from(
                (0..36).map(|j| j as f64).collect::<Vec<f64>>(),
            )),
        );
        assert_float_eq!(brane.grid[0], 1.608919, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.398250, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.0, abs <= EPSILON);
    }
}
