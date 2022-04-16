#[allow(unused_imports)]
use crate::carto::{brane::Brane, colour as clr, render::Renderable};
use crate::climate::{
    chart::{Chart, Zone},
    geology::{
        altitude_with_ocean, bedrock_level, bedrock_vege, continentality, ocean_tiles,
        INIT_OCEAN_LEVEL,
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
}

const YEAR_LEN: usize = 16;
const RES_SMALL: usize = 144;

fn chartise(year: &Vec<Month>) -> Brane<Chart> {
    let resolution = year
        .get(0)
        .expect("cannot process an empty year")
        .temp
        .resolution;
    let mut charts = (0..resolution.pow(2))
        .into_iter()
        .map(|_| Chart::new())
        .collect::<Vec<Chart>>();
    for month in &year[match year.len().checked_sub(YEAR_LEN) {
        Some(v) => v,
        None => 0,
    }..]
    {
        for jndex in 0..resolution.pow(2) {
            charts[jndex].push(
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
    let mut altitude_small = altitude.upscale(RES_SMALL);
    let continentality_small = continentality(&altitude_small, INIT_OCEAN_LEVEL);
    let mut vege_small = bedrock_vege(&altitude_small);

    altitude_small.variable = "alt".to_string();
    altitude_small.render(clr::TopographyInk::new(0.25));

    // # dry run
    let mut dry = Vec::<Month>::new();
    for sol in 0..(YEAR_LEN + 1) {
        dry.push(simulate_month(
            sol as f64 / YEAR_LEN as f64,
            INIT_OCEAN_LEVEL,
            &altitude_small,
            &continentality_small,
            &vege_small,
        ));
        let charts = chartise(&dry);
        vege_small = veges(&charts, &altitude_small, INIT_OCEAN_LEVEL);
    }

    /*
    let charts = chartise(&dry);
    let mut a = Brane::from(
        (0..charts.resolution.pow(2))
            .into_par_iter()
            .map(|j| charts.grid[j].aridity())
            .collect::<Vec<f64>>(),
    );
    a.variable = "aridity".to_string();
    a.stats();
    a.render(clr::HueInk::new(0.12, 0.92));
    let mut s = Brane::from(
        (0..charts.resolution.pow(2))
            .into_par_iter()
            .map(|j| charts.grid[j].swing())
            .collect::<Vec<f64>>(),
    );
    s.variable = "swing".to_string();
    s.stats();
    s.render(clr::BiHueInk::new(0.54, 0.04, 0.92));
    let mut tmin = Brane::from(
        (0..charts.resolution.pow(2))
            .into_par_iter()
            .map(|j| charts.grid[j].tmin())
            .collect::<Vec<f64>>(),
    );
    tmin.variable = "tmin".to_string();
    tmin.stats();
    tmin.render(clr::CelciusInk);
    let mut tmax = Brane::from(
        (0..charts.resolution.pow(2))
            .into_par_iter()
            .map(|j| charts.grid[j].tmax())
            .collect::<Vec<f64>>(),
    );
    tmax.variable = "tmax".to_string();
    tmax.stats();
    tmax.render(clr::CelciusInk);
    */

    vege_small.variable = "vege".to_string();
    vege_small.render(clr::KoppenInk);

    // erosion
    // eroded run
}
