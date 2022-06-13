use crate::{
    carto::brane::Brane,
    climate::{
        chart::{Chart, Zone},
        geology::{ocean, ocean_tiles},
        simulation::simulate,
        vegetation::{habitability, Vege},
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

    pub fn score(&self) -> f64 {
        let count: f64 = self.altitude.resolution.pow(2) as f64;
        let land_cover = 1.0
            - ocean_tiles(&self.altitude, ocean(&self.altitude))
                .grid
                .into_iter()
                .filter(|b| *b)
                .map(|_| 1)
                .sum::<usize>() as f64
                * count.recip();
        let habitability = self
            .vege()
            .grid
            .into_iter()
            .map(|zone| habitability(zone))
            .sum::<f64>()
            * count.recip();
        let r = ((land_cover - 0.64).powi(2) + 2. * (habitability - 0.24).powi(2)).sqrt();
        println!("{:.4} {:.4} {:.4}", land_cover, habitability, r);
        r
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;
}
