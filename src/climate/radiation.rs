use crate::{
    cartography::{
        brane::Brane,
        hexagonos::{Gon, PreGon},
    },
    util::{
        constants::*,
        diffusion::{diffusion_medium, Medium},
    },
};
use geo::Coordinate;
use log::info;
use nalgebra::Vector3;
use ordered_float::OrderedFloat;
use petgraph::graph::{Graph, NodeIndex};
use rayon::prelude::*;
use std::collections::HashMap;

/* # insolation */

fn vector_elevation(point: &Coordinate<f64>, elevation: f64) -> Vector3<f64> {
    let cpoint = point.centre();
    Vector3::new(cpoint.x, cpoint.y, elevation)
}

fn insolation_calculate_sun(point: &Coordinate<f64>, sun: &Coordinate<f64>) -> f64 {
    //turns out, the influence of both elevation and slope is negligable
    let sunward = vector_elevation(&sun, 1.0) - vector_elevation(&point, 0.0);
    sunward.norm().powi(-2) * sunward.dot(&Vector3::new(0.0, 0.0, 1.0)) / sunward.norm()
}

fn insolation_calculate_point(point: &Coordinate<f64>) -> f64 {
    let suns = point.find().ball(SOL_DETAIL);
    suns.iter()
        .map(|sun| Coordinate {
            x: sun.x as f64,
            y: sun.y as f64,
        })
        .map(|sun| insolation_calculate_sun(&point, &sun))
        .sum::<f64>()
}

/// calculate insolation – the amount of radiation reaching the surface over a single day
pub fn insolation_calculate(resolution: usize) -> Brane<f64> {
    info!("calculating insolation map");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| insolation_calculate_point(&point))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "insolation".to_string();
    brane
}

/* # temperature */

/// initialise temperature to a given value in degrees Kelvin
pub fn temperature_initialise(resolution: usize, insolation: &Brane<f64>) -> Brane<f64> {
    info!("initialising temperature");
    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| INIT_TEMP + SOL_POWER * insolation.get(&point))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "temperature".to_string();
    brane
}

/* ## diffusion */

fn choose(j: usize) -> Medium {
    match j % 6 {
        0 => Medium::Air,
        _ => Medium::Ocean,
    }
}

/// calculate temperature diffusion
pub fn temperature_diffuse(temperature: &mut Brane<f64>, surface: &Brane<u8>) {
    info!("calculating temperature diffusion");

    for j in 0..temperature.resolution * 18 {
        temperature.grid = temperature
            .into_par_iter()
            .map(|point| diffusion_medium(&point, choose(j), &temperature, &surface))
            .collect::<Vec<f64>>();
    }
}

/// calculate average temperature
pub fn temperature_calculate(
    resolution: usize,
    insolation: &Brane<f64>,
    surface: &Brane<u8>,
) -> Brane<f64> {
    let mut temperature = temperature_initialise(resolution, &insolation);
    temperature_diffuse(&mut temperature, &surface);
    temperature
}

/* # pressure */

#[allow(dead_code)]
fn pressure_elevation(pressure: f64, elevation: f64, temperature: f64) -> f64 {
    pressure * (LAPSE_CONST * elevation * temperature.recip()).exp()
}

/// calculate pressure at ocean level
pub fn pressure_calculate(resolution: usize, temperature: &Brane<f64>) -> Brane<f64> {
    info!("calculating pressure at ocean level");
    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| {
                temperature
                    .get(&point)
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
    for point in pressure.exact_iter() {
        let here = graph.add_node(point);
        nodes.insert(point, here);
    }
    for point in pressure.exact_iter() {
        let minbr = *pressure
            .exact_ambit(&point)
            .iter()
            .min_by_key(|nbr| OrderedFloat(pressure.read(&nbr)))
            .unwrap();
        let dif = pressure.read(&point) - pressure.read(&minbr);
        if dif > 0.0 {
            graph.add_edge(nodes[&point], nodes[&minbr], dif.recip().log10() * 0.1);
        } else {
            roots.push(nodes[&point]);
        }
    }
    (graph, roots)
}

/*

// this may be useful when seasons come around

/* ## absorbtion */

fn temperature_absorb_point(
    point: &Coordinate<f64>,
    temperature: &Brane<f64>,
    insolation: &Brane<f64>,
    albedo: &Brane<f64>,
    capacitance: &Brane<f64>,
) -> f64 {
    temperature.get(&point)
        + TIME_LOCAL * insolation.get(&point) * (1.0 - albedo.get(&point)) / capacitance.get(&point)
}

pub fn temperature_absorb(
    temperature: &mut Brane<f64>,
    insolation: &Brane<f64>,
    albedo: &Brane<f64>,
    capacitance: &Brane<f64>,
) {
    info!("calculating temperature increase from absorbtion");

    temperature.grid = temperature
        .into_par_iter()
        .map(|point| {
            temperature_absorb_point(&point, &temperature, &insolation, &albedo, &capacitance)
        })
        .collect::<Vec<f64>>();
}

/* ## radiation */

fn temperature_radiate_point(
    point: &Coordinate<f64>,
    temperature: &Brane<f64>,
    capacitance: &Brane<f64>,
) -> f64 {
    let current = temperature.get(&point);
    current - TIME_LOCAL * SB_CNST * current.powi(4) / capacitance.get(&point)
}

pub fn temperature_radiate(temperature: &mut Brane<f64>, capacitance: &Brane<f64>) {
    info!("calculating temperature decrease from radiation");

    temperature.grid = temperature
        .into_par_iter()
        .map(|point| temperature_radiate_point(&point, &temperature, &capacitance))
        .collect::<Vec<f64>>();
}

*/
