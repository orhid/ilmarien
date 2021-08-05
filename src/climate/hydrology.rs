use crate::imaging::cartography::Brane;
use geo_types::Coordinate;
use log::info;
use rayon::prelude::*;

/* initialise */

fn ocean_initialise_point(point: &Coordinate<f64>, elevation: &Brane<f64>) -> f64 {
    let pelev = elevation.get(&point);
    if pelev < 0.25 {
        0.25 - pelev
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
