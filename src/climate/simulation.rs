#[allow(unused_imports)]
use crate::carto::{brane::Brane, colour as clr, render::Renderable};
use crate::climate::{
    chart::{Chart, Zone},
    geology::{
        altitude_with_ocean, bedrock_level, bedrock_vege, continentality, ocean, ocean_tiles,
    },
    hydrology::{evaporation, potential_evaporation, rainfall},
    radiation::{temperature, temperature_oceanlv, wind},
    vegetation::Vege,
};
use rayon::prelude::*;

pub struct Month {
    pub temp: Brane<f64>,
    pub rain: Brane<f64>,
    pub pevt: Brane<f64>,
}

impl Month {
    pub fn new(temp: Brane<f64>, rain: Brane<f64>, pevt: Brane<f64>) -> Self {
        Self { temp, rain, pevt }
    }

    pub fn upscale(self, altitude: &Brane<f64>) -> Self {
        let resolution = altitude.resolution;
        Self::new(
            self.temp.upscale(resolution),
            self.rain.upscale(resolution),
            self.pevt.upscale(resolution),
        )
    }
}

const YEAR_LEN: usize = 6;
const RES_SMALL: usize = 144;

/*
fn total_rain(year: &[Month]) -> Brane<f64> {
    Brane::from(
        (0..year
            .get(0)
            .expect("cannot process an empty year")
            .temp
            .resolution
            .pow(2))
            .into_iter()
            .map(|jndex| {
                year[year.len().saturating_sub(YEAR_LEN)..]
                    .iter()
                    .map(|month| month.rain.grid[jndex])
                    .sum::<f64>()
                    * (YEAR_LEN as f64).recip()
                    * 24.0
            })
            .collect::<Vec<f64>>(),
    )
}
*/

fn predict(mut year_small: Vec<Month>, altitude: &Brane<f64>) -> Vec<Month> {
    let mut year = Vec::<Month>::new();
    for month in year_small
        .drain(year_small.len().saturating_sub(YEAR_LEN)..)
        .collect::<Vec<Month>>()
        .into_iter()
    {
        //month.upscale()
        year.push(month.upscale(altitude));
    }
    year
}

fn chartise(year: &[Month]) -> Brane<Chart> {
    let resolution = year
        .get(0)
        .expect("cannot process an empty year")
        .temp
        .resolution;
    let mut charts = (0..resolution.pow(2))
        .into_iter()
        .map(|_| Chart::new())
        .collect::<Vec<Chart>>();
    for month in &year[year.len().saturating_sub(YEAR_LEN)..] {
        for (jndex, chart) in charts.iter_mut().enumerate().take(resolution.pow(2)) {
            chart.push(
                month.temp.grid[jndex],
                month.rain.grid[jndex],
                month.pevt.grid[jndex],
            );
        }
    }
    Brane::from(charts)
}

fn veges(charts: &Brane<Chart>, altitude: &Brane<f64>, ocean: f64) -> Brane<Option<Vege>> {
    let ocean_tiles = ocean_tiles(altitude, ocean);
    Brane::from(
        (0..charts.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                if ocean_tiles.grid[j] {
                    None
                } else {
                    Some(Vege::from(&Zone::from(&charts.grid[j])))
                }
            })
            .collect::<Vec<Option<Vege>>>(),
    )
}

fn simulate_month(
    sol: f64,
    ocean: f64,
    altitude: &Brane<f64>,
    continentality: &Brane<f64>,
    vege: &Brane<Option<Vege>>,
) -> Month {
    let tmp_olv = temperature_oceanlv(sol, continentality);
    let alt_wo = altitude_with_ocean(altitude, ocean);
    let temp = temperature(&tmp_olv, &alt_wo);
    let pevt = potential_evaporation(&temp);
    Month::new(
        temp,
        rainfall(&alt_wo, &evaporation(&pevt, vege), &wind(&tmp_olv)),
        pevt,
    )
}

pub fn simulate(resolution: usize, seed: u32) {
    // # generate bedrock
    let altitude = bedrock_level(resolution, seed);
    let ocean_level = ocean(&altitude);
    let altitude_small = altitude.upscale(RES_SMALL);
    let continentality_small = continentality(&altitude_small, ocean_level);
    let mut vege_small = bedrock_vege(&altitude_small);

    // # dry run
    let mut year_small = Vec::<Month>::new();
    for sol in 0..(YEAR_LEN) {
        year_small.push(simulate_month(
            sol as f64 / YEAR_LEN as f64,
            ocean_level,
            &altitude_small,
            &continentality_small,
            &vege_small,
        ));
        vege_small = veges(&chartise(&year_small), &altitude_small, ocean_level);
    }

    //vege_small.variable = format!("vege-{}", seed);
    //vege_small.render(clr::KoppenInk);

    /*
    // # erode
    erode(&mut altitude, &total_rain(&year_small).upscale(resolution));
    let ocean_level = ocean(&altitude);
    let mut altitude_small = altitude.upscale(RES_SMALL);
    let continentality_small = continentality(&altitude_small, ocean_level);
    */

    // # eroded run
    // let mut year_small = Vec::<Month>::new(); // use results from dry run
    for sol in 0..(2 * YEAR_LEN + 1) {
        year_small.push(simulate_month(
            sol as f64 / YEAR_LEN as f64,
            ocean_level,
            &altitude_small,
            &continentality_small,
            &vege_small,
        ));
        vege_small = veges(&chartise(&year_small), &altitude_small, ocean_level);
    }

    // # upscale
    // upscale the last YEAR_LEN from year_small and use it to calculate veges on the full resolution map
    //let continentality = continentality(&altitude, ocean_level);
    let year = predict(year_small, &altitude);
    let mut vege = veges(&chartise(&year), &altitude, ocean_level);
    vege.variable = format!("vege-{}", seed);
    vege.render(clr::KoppenInk);

    // # finish
    // score the generated cosmos
    // save the relevant bits of information
}
