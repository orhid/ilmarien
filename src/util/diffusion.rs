use crate::cartography::brane::Brane;
use crate::climate::surface::{decode, Surface};
use geo::Coordinate;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Medium {
    Air,
    Ocean,
}

pub fn diffusion_medium(
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

pub fn diffusion_level(point: &Coordinate<f64>, fluid: &Brane<f64>, level: &Brane<f64>) -> f64 {
    let herelev = level.get(&point);
    let mut ambit = fluid.ambit(&point);
    ambit = ambit
        .into_iter()
        .filter(|gon| (level.get(&gon) - herelev).abs() < 0.032)
        .collect();
    let len = ambit.len() as f64;
    if len > 0.0 {
        ambit.into_iter().map(|gon| fluid.get(&gon)).sum::<f64>() / len
    } else {
        fluid.get(&point)
    }
}

// this should be tested somehow
