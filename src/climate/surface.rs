use crate::imaging::cartography as crt;
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

fn encode(surface: Surface) -> u16 {
    match surface {
        Surface::Water => 0,
        Surface::Ice => 1,
        Surface::Snow => 2,
        Surface::Stone => 3,
    }
}

pub fn decode(value: u16) -> Surface {
    match value {
        0 => Surface::Water,
        1 => Surface::Ice,
        2 => Surface::Snow,
        3 => Surface::Stone,
        _ => panic!(),
    }
}

fn surface_calculate_point(point: &Coordinate<f64>, water: &crt::Brane) -> Surface {
    if water.find_value(&point) > 0.0 {
        Surface::Water
    } else {
        Surface::Stone
    }
}

pub fn surface_calculate(resolution: usize, water: &crt::Brane) -> crt::Brane {
    //! calculate surface type

    info!("calculating surface type");

    let mut brane = crt::new("surface".to_string(), resolution);
    brane.engrid_exact(
        brane
            .into_par_iter()
            .map(|point| encode(surface_calculate_point(&point, &water)))
            .collect(),
    );
    brane
}

/* albedo */

fn albedo_calculate_point(point: &Coordinate<f64>, surface: &crt::Brane) -> f64 {
    match decode(surface.get_exact(&surface.find(&point))) {
        Surface::Water => 0.06,
        Surface::Ice => 0.36,
        Surface::Snow => 0.9,
        Surface::Stone => 0.32,
    }
}

pub fn albedo_calculate(resolution: usize, surface: &crt::Brane) -> crt::Brane {
    //! calculate surface albedo

    info!("calculating surface albedo");

    let mut brane = crt::new("albedo".to_string(), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| albedo_calculate_point(&point, &surface))
            .collect(),
    );
    brane
}
