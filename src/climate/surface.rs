use crate::imaging::cartography::Brane;
use geo_types::Coordinate;
use log::info;
use rayon::prelude::*;

/* surface types */

#[derive(PartialEq)]
pub enum Surface {
    Water,
    Ice,
    Snow,
    Stone,
}

fn encode(surface: Surface) -> u8 {
    match surface {
        Surface::Water => 0,
        Surface::Ice => 1,
        Surface::Snow => 2,
        Surface::Stone => 3,
    }
}

pub fn decode(value: u8) -> Surface {
    match value {
        0 => Surface::Water,
        1 => Surface::Ice,
        2 => Surface::Snow,
        3 => Surface::Stone,
        _ => panic!(),
    }
}

fn surface_calculate_point(point: &Coordinate<f64>, ocean: &Brane<f64>) -> Surface {
    if ocean.get(&point) > 0.0 {
        Surface::Water
    } else {
        Surface::Stone
    }
}

/// calculate surface type
pub fn surface_calculate(resolution: usize, ocean: &Brane<f64>) -> Brane<u8> {
    info!("calculating surface type");

    let mut brane = Brane::from(
        Brane::<u8>::vec_par_iter(resolution)
            .map(|point| encode(surface_calculate_point(&point, &ocean)))
            .collect::<Vec<u8>>(),
    );
    brane.variable = "surface".to_string();
    brane
}

/*

// this may become useful later

/* albedo */

fn albedo_calculate_point(point: &Coordinate<f64>, surface: &Brane<u8>) -> f64 {
    match decode(surface.get(&point)) {
        Surface::Water => 0.06,
        Surface::Ice => 0.36,
        Surface::Snow => 0.9,
        Surface::Stone => 0.32,
    }
}

/// calculate surface albedo
pub fn albedo_calculate(resolution: usize, surface: &Brane<u8>) -> Brane<f64> {
    info!("calculating surface albedo");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| albedo_calculate_point(&point, &surface))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "albedo".to_string();
    brane
}

/* capacitance */

fn capacitance_calculate_point(point: &Coordinate<f64>, surface: &Brane<u8>) -> f64 {
    match decode(surface.get(&point)) {
        // heat capacitance in J*g^-1*K^-1
        Surface::Water => 4.1813,
        Surface::Ice => 2.05,
        Surface::Snow => 2.05,
        Surface::Stone => 0.79,
    }
}

/// calculate surface albedo
pub fn capacitance_calculate(resolution: usize, surface: &Brane<u8>) -> Brane<f64> {
    info!("calculating surface albedo");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| capacitance_calculate_point(&point, &surface))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "capacitance".to_string();
    brane
}

*/
