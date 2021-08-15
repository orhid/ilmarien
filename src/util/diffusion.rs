use crate::climate::surface::{decode, Surface};
use crate::imaging::cartography::Brane;
use geo_types::Coordinate;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Medium {
    Air,
    Ocean,
}

pub fn diffusion_calculate_point(
    point: &Coordinate<f64>,
    medium: Medium,
    fluid: &Brane<f64>,
    surface: &Brane<u8>,
) -> f64 {
    if medium == Medium::Air || decode(surface.get(&point)) == Surface::Water {
        let mut ambit = fluid.ambit(&point);
        if medium == Medium::Ocean {
            ambit = ambit
                .into_iter()
                .filter(|gon| decode(surface.get(&gon)) == Surface::Water)
                .collect();
        }
        let len = ambit.len() as f64;
        if len > 0.0 {
            ambit.into_iter().map(|gon| fluid.get(&gon)).sum::<f64>() / len
        } else {
            fluid.get(&point)
        }
    } else {
        fluid.get(&point)
    }
}

// this should be tested somehow
