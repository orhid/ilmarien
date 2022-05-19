use crate::{
    carto::brane::Brane,
    climate::{
        chart::{Chart, Zone},
        geology::{ocean, ocean_tiles},
        simulation::simulate,
        vegetation::Vege,
    },
};
//use log::trace;
use rayon::prelude::*;

pub struct Cosmos {
    pub altitude: Brane<f64>,
    pub charts: Brane<Chart>,
}

impl Cosmos {
    /// initialise cosmos with given bedrock brane
    pub fn new(altitude: Brane<f64>, charts: Brane<Chart>) -> Self {
        Self { altitude, charts }
    }

    pub fn sim_new(resolution: usize, seed: u32) -> Self {
        let (altitude, charts) = simulate(resolution, seed);
        Self { altitude, charts }
    }

    pub fn vege(&self) -> Brane<Option<Vege>> {
        let ocean_tiles = ocean_tiles(&self.altitude, ocean(&self.altitude));
        Brane::from(
            (0..self.charts.resolution.pow(2))
                .into_par_iter()
                .map(|j| {
                    if ocean_tiles.grid[j] {
                        None
                    } else {
                        Some(Vege::from(&Zone::from(&self.charts.grid[j])))
                    }
                })
                .collect::<Vec<Option<Vege>>>(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;
}
