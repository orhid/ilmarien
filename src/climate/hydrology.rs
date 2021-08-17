use crate::cartography::brane::Brane;
use crate::climate::{
    radiation::pressure_gradient,
    surface::{decode, Surface},
};
use crate::util::{constants::*, diffusion::diffusion_level};
use geo::Coordinate;
use log::info;
use ordered_float::OrderedFloat as Orf;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;
use std::cmp::{max, min};

/* # initialise */

fn ocean_initialise_point(point: &Coordinate<f64>, elevation: &Brane<f64>) -> f64 {
    let pelev = elevation.get(&point);
    if pelev < INIT_OCEAN_LEVEL {
        INIT_OCEAN_LEVEL - pelev
    } else {
        0.0
    }
}

/// calculate initial ocean level
pub fn ocean_initialise(resolution: usize, elevation: &Brane<f64>) -> Brane<f64> {
    info!("calculating initial ocean level");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| ocean_initialise_point(&point, &elevation))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "ocean".to_string();
    brane
}

/* # rainfall */

/* ## evaporation */

fn evaporation_rate(temperature: f64, pressure: f64) -> f64 {
    (temperature * pressure.recip()).mul_add(108f64.recip(), -2.2)
}

fn evaporation_calculate_point(
    point: &Coordinate<f64>,
    surface_type: &Brane<u8>,
    temperature: &Brane<f64>,
    pressure: &Brane<f64>,
) -> f64 {
    match decode(surface_type.get(&point)) {
        Surface::Water => evaporation_rate(temperature.get(&point), pressure.get(&point)),
        Surface::Ice | Surface::Snow => {
            0.24 * evaporation_rate(temperature.get(&point), pressure.get(&point))
        }
        _ => 0.0,
    }
}

/// calculate evaporation rate
pub fn evaporation_calculate(
    resolution: usize,
    surface_type: &Brane<u8>,
    temperature: &Brane<f64>,
    pressure: &Brane<f64>,
) -> Brane<f64> {
    info!("calculating evaporation rate");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| {
                evaporation_calculate_point(&point, &surface_type, &temperature, &pressure)
            })
            .collect::<Vec<f64>>(),
    );
    brane.variable = "evaporation".to_string();
    brane
}

fn rainfall_node(
    target_level: f64,
    node: NodeIndex,
    gradient: &Graph<Coordinate<i32>, f64>,
    evaporation: &Brane<f64>,
    elevation: &Brane<f64>,
    rainfall: &mut Brane<f64>,
) -> f64 {
    let point = &gradient[node];
    let level = elevation.get(&evaporation.cast(&point));
    let moisture = evaporation.read(&point)
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
    rainfall.insert(&point, frac);
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
            .into_par_iter()
            .map(|point| diffusion_level(&point, &rainfall, &elevation))
            .collect::<Vec<f64>>();
    }
    rainfall.variable = "rainfall".to_string();
    rainfall
}
