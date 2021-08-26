use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
    },
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

fn reflow_filter(here_lev: f64, nbr_lev: f64, here_sur: Fabric, nbr_sur: Fabric) -> bool {
    if here_sur == Fabric::Water {
        if nbr_sur == Fabric::Water {
            true
        } else {
            here_lev > nbr_lev
        }
    } else {
        if nbr_sur == Fabric::Water {
            here_lev < nbr_lev
        } else {
            false
        }
    }
}

pub fn reflow(datum: &DatumZa, fluid: &Brane<f64>, surface: &Brane<Fabric>) -> f64 {
    let herelev = fluid.read(&datum);
    let heresur = surface.read(&datum);
    let mut ambit = fluid.ambit_exact(&datum);
    ambit = ambit
        .into_iter()
        .filter(|gon| reflow_filter(herelev, fluid.read(&gon), heresur, surface.read(&gon)))
        .collect();
    let len = ambit.len() as f64;
    if len > 0.0 {
        ambit.into_iter().map(|gon| fluid.read(&gon)).sum::<f64>() / len
    } else {
        herelev
    }
}

// this should be tested somehow
