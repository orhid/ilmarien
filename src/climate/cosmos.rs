use crate::{
    carto::{
        brane::Brane,
        datum::DatumZa,
        honeycomb::{ball_volume, HoneyCellToroidal},
    },
    climate::{
        chart::{Chart, Zone},
        hydrology::{evaporation, evapotranspiration, rainfall},
        radiation::{insolation, lapse, temperature_update, wind},
        vegetation::Vege,
    },
    vars::*,
};
use log::trace;
use rayon::prelude::*;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Cell {
    pub altitude: f64,
    chart: Chart,
}

impl Cell {
    /// initialise cell with given altitude
    pub fn new(altitude: f64) -> Self {
        Self {
            altitude,
            chart: Chart::new(),
        }
    }

    fn is_ocean(&self, ocean: f64) -> bool {
        self.altitude < ocean
    }

    /// altitude with ocean
    pub fn altitude(&self, ocean: f64) -> f64 {
        if self.is_ocean(ocean) {
            ocean
        } else {
            self.altitude
        }
    }
}

pub struct Cosmos {
    pub brane: Brane<Cell>,
    pub ocean: f64,
}

impl Cosmos {
    /// initialise cosmos with given bedrock brane
    pub fn new(bedrock: &Brane<f64>) -> Self {
        Self {
            brane: Brane::from(
                (0..bedrock.resolution.pow(2))
                    .into_par_iter()
                    .map(|j| Cell::new(bedrock.grid[j]))
                    .collect::<Vec<Cell>>(),
            ),
            ocean: INIT_OCEAN_LEVEL,
        }
    }

    /// find ocean tiles
    fn ocean_tiles(&self) -> Brane<bool> {
        Brane::from(
            (0..self.brane.resolution.pow(2))
                .into_par_iter()
                .map(|j| {
                    DatumZa::enravel(j, self.brane.resolution)
                        .ball_toroidal(
                            self.brane.resolution.div_euclid(108) as i32,
                            self.brane.resolution as i32,
                        )
                        .iter()
                        .map(|datum| {
                            (self.ocean
                                - self.brane.grid[datum.unravel(self.brane.resolution)].altitude)
                                .signum()
                        })
                        .sum::<f64>()
                        > ball_volume(self.brane.resolution.div_euclid(108) as i32) as f64 * 0.72
                })
                .collect::<Vec<bool>>(),
        )
    }

    /// find mountain tiles
    fn mountain_tiles(&self) -> Brane<bool> {
        Brane::from(
            (0..self.brane.resolution.pow(2))
                .into_par_iter()
                .map(|j| self.brane.grid[j].altitude > self.ocean + 0.12)
                .collect::<Vec<bool>>(),
        )
    }

    /// find distance to closest ocean, going around mountains
    fn continentality(&self) -> Brane<f64> {
        trace!("calculating continentality coefficients");
        let step = 4.0 / self.brane.resolution as f64;
        let mut continentality = Brane::from(
            self.mountain_tiles()
                .grid
                .into_par_iter()
                .map(|b| if b { 1.0 } else { f64::NAN })
                .collect::<Vec<f64>>(),
        );

        // preopoulate oceans with zeros
        let mut ocean_datums = VecDeque::new();
        let ocean_tiles = self.ocean_tiles();
        for index in 0..self.brane.resolution.pow(2) {
            if ocean_tiles.grid[index] {
                continentality.grid[index] = 0.0;
                ocean_datums.push_back(DatumZa::enravel(index, self.brane.resolution));
            }
        }

        // flood fill from ocean datums
        while !ocean_datums.is_empty() {
            let here = ocean_datums.pop_front().unwrap();
            for datum in here.ambit_toroidal(self.brane.resolution as i32) {
                let index = datum.unravel(self.brane.resolution);
                if continentality.grid[index].is_nan() {
                    continentality.grid[index] =
                        continentality.grid[here.unravel(self.brane.resolution)] + step;
                    ocean_datums.push_back(datum);
                }
            }
        }

        // check if any tiles were left stranded inside mountain regions
        for j in 0..self.brane.resolution.pow(2) {
            if continentality.grid[j].is_nan() {
                continentality.grid[j] = 1.0
            }
        }
        continentality
    }

    /// calculate the altitude model, with oceans
    fn altitude(&self) -> Brane<f64> {
        Brane::from(
            (0..self.brane.resolution.pow(2))
                .into_par_iter()
                .map(|j| self.brane.grid[j].altitude(self.ocean))
                .collect::<Vec<f64>>(),
        )
    }

    /// add data to climate chart
    fn push(
        &mut self,
        temperature: &Brane<f64>,
        rainfall: &Brane<f64>,
        evaporation: &Brane<f64>,
        cycle: usize,
    ) {
        for j in 0..self.brane.resolution.pow(2) {
            let altitude = self.brane.grid[j].altitude(self.ocean);
            self.brane.grid[j].chart.push(
                temperature.grid[j] - lapse(altitude),
                rainfall.grid[j],
                evaporation.grid[j],
                cycle,
            );
        }
    }

    /// calculate vegetation type
    pub fn vege(&self) -> Brane<Option<Vege>> {
        Brane::from(
            (0..self.brane.resolution.pow(2))
                .into_par_iter()
                .map(|j| {
                    let cell = &self.brane.grid[j];
                    if cell.is_ocean(self.ocean) {
                        None
                    } else {
                        Some(Zone::from(&cell.chart).vege())
                    }
                })
                .collect::<Vec<Option<Vege>>>(),
        )
    }

    /// run a simulation for a month
    fn sim_month(
        &mut self,
        sol: f64,
        months: usize,
        temperature: &mut Brane<f64>,
        altitude: &Brane<f64>,
        continentality: &Brane<f64>,
    ) {
        temperature_update(
            &insolation(self.brane.resolution, sol),
            temperature,
            continentality,
        );
        let evaporation = evaporation(temperature);
        self.push(
            &temperature.clone().mul_add(TEMP_RANGE, -6.0),
            &rainfall(
                altitude,
                &evapotranspiration(&evaporation, &self.vege()),
                &wind(&temperature.upscale(RAIN_RES)),
            ),
            &evaporation,
            months,
        );
    }

    /// run a simulation for a year
    fn sim_year(
        &mut self,
        months: usize,
        temperature: &mut Brane<f64>,
        altitude: &Brane<f64>,
        continentality: &Brane<f64>,
    ) {
        for sol in (0..months)
            .map(|month| SOL_DEV * (std::f64::consts::TAU * month as f64 / months as f64).sin())
        {
            self.sim_month(sol, months, temperature, altitude, continentality);
        }
    }

    /// run a climate simulation
    pub fn sim_climate(&mut self, years: usize, months: usize) {
        let altitude = self.altitude();
        let continentality = self.continentality();
        let mut temperature = insolation(self.brane.resolution, 0.0);
        for _ in 0..years {
            self.sim_year(months, &mut temperature, &altitude, &continentality);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    #[test]
    fn cell_altitude() {
        let cell = Cell::new(0.5);
        assert_float_eq!(cell.altitude(0.3), 0.5, abs <= EPSILON);
        assert_float_eq!(cell.altitude(0.7), 0.7, abs <= EPSILON);
    }

    #[test]
    fn cosmos_new() {
        let cosmos = Cosmos::new(&Brane::from(vec![0.0, 0.25, 0.5, 0.75]));
        assert_eq!(cosmos.brane.grid.len(), 4);
        let altitude = cosmos.altitude();
        assert_float_eq!(altitude.grid[0], 0.25, abs <= EPSILON); // ocean
        assert_float_eq!(altitude.grid[1], 0.25, abs <= EPSILON);
        assert_float_eq!(altitude.grid[2], 0.5, abs <= EPSILON);
    }

    #[test]
    fn continentality_values() {
        let brane = Cosmos::new(&Brane::from(
            (0..36).map(|j| j as f64 / 36.0).collect::<Vec<f64>>(),
        ))
        .continentality();
        assert_float_eq!(brane.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[12], 0.666666, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 1.0, abs <= EPSILON);
    }

    #[test]
    fn vege_values() {
        let cosmos = Cosmos::new(&Brane::from(
            (0..36).map(|j| j as f64 / 36.0).collect::<Vec<f64>>(),
        ));
        let brane = cosmos.vege();
        assert_eq!(brane.grid[0], None);
        assert_eq!(*brane.grid[24].as_ref().unwrap(), Vege::Stone);
    }
}
