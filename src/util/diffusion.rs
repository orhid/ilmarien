use crate::{
    carto::{brane::Brane, datum::DatumRe},
    climate::cosmos::Fabric,
};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Medium {
    Air,
    Ocean,
}

pub fn diffuse_medium(
    datum: &DatumRe,
    medium: Medium,
    fluid: &Brane<f64>,
    surface: &Brane<Fabric>,
) -> f64 {
    if medium == Medium::Air || surface.get(&datum) == Fabric::Water {
        let mut ambit = fluid.ambit(&datum);
        if medium == Medium::Ocean {
            ambit = ambit
                .into_iter()
                .filter(|gon| surface.get(&gon) == Fabric::Water)
                .collect();
        }
        let len = ambit.len() as f64;
        if len > 0.0 {
            ambit.into_iter().map(|gon| fluid.get(&gon)).sum::<f64>() / len
        } else {
            fluid.get(&datum)
        }
    } else {
        fluid.get(&datum)
    }
}

pub fn diffuse_level(datum: &DatumRe, fluid: &Brane<f64>, level: &Brane<f64>) -> f64 {
    let herelev = level.get(&datum);
    let mut ambit = fluid.ambit(&datum);
    ambit = ambit
        .into_iter()
        .filter(|gon| (level.get(&gon) - herelev).abs() < 0.032)
        .collect();
    let len = ambit.len() as f64;
    if len > 0.0 {
        ambit.into_iter().map(|gon| fluid.get(&gon)).sum::<f64>() / len
    } else {
        fluid.get(&datum)
    }
}

// this should be tested somehow
