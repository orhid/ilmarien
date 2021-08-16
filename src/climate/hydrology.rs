use crate::cartography::brane::Brane;
use crate::util::constants::*;
use geo::Coordinate;
use log::info;
use rayon::prelude::*;

/* initialise */

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

/* rainfall */

/*
/// simulate the amount of rainfall reaching the surface
pub fn rainfall_simulate(
    resolution: usize,
    elevation: &Brane<f64>,
    ocean: &mut Brane<f64>,
    heat: &Brane<f64>,
) -> Brane<u8> {
    info!("simulating rainfall");

    // this is all wrong
    let mut brane = Brane::from(
        Brane::<u8>::vec_par_iter(resolution)
            .map(|point| rainfall_calculate_point(&point, &evaporation))
            .collect::<Vec<u8>>(),
    );
    brane.variable = "rainfall".to_string();
    brane
}
*/
