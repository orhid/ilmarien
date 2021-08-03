use crate::imaging::cartography as crt;
use geo_types::Coordinate;
use log::info;
use rayon::prelude::*;

/* initialise */

fn water_initialise_point(point: &Coordinate<f64>, elevation: &crt::Brane) -> f64 {
    let pelev = elevation.find_value(&point);
    if pelev < 0.25 {
        0.25 - pelev
    } else {
        0.0
    }
}

pub fn water_initialise(resolution: usize, elevation: &crt::Brane) -> crt::Brane {
    //! calculate initial ocean level

    info!("calculating initial ocean level");

    let mut brane = crt::new("water".to_string(), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| water_initialise_point(&point, &elevation))
            .collect(),
    );
    brane
}
