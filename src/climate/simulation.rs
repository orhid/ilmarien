#[allow(unused_imports)]
//use crate::carto::{colour as clr, render::Renderable};
use crate::{
    carto::brane::{Brane, Resolution},
    climate::{
        chart::{Chart, Zone},
        geology::{altitude_above_ocean_level, bedrock, erode, ocean_level, ocean_tiles},
        hydrology::{continentality, evapotranspiration_potential, rainfall},
        radiation::{temperature_at_altitude, temperature_at_ocean_level, temperature_average},
        regression::predict_brane,
        vegetation::Vege,
    },
    units::{Elevation, Precipitation, Temperature, Unit},
};
use log::trace;

/* # months */

pub struct Month {
    pub temp: Brane<Temperature>,
    pub rain: Brane<Precipitation>,
    pub pevt: Brane<Precipitation>,
}

impl Month {
    pub fn new(
        temp: Brane<Temperature>,
        rain: Brane<Precipitation>,
        pevt: Brane<Precipitation>,
    ) -> Self {
        Self { temp, rain, pevt }
    }

    pub fn upscale(
        self,
        altitude_smol: &Brane<Elevation>,
        continentality_smol: &Brane<f64>,
        altitude: &Brane<Elevation>,
        continentality: &Brane<f64>,
    ) -> Self {
        let temperature = predict_brane(
            &self.temp.release(),
            altitude_smol,
            continentality_smol,
            altitude,
            continentality,
        )
        .operate_by_value(Temperature::confine);
        Self::new(
            temperature.clone(),
            self.rain.upscale(altitude.resolution),
            evapotranspiration_potential(&temperature),
        )
    }
}

fn simulate_month(
    solar_time: f64,
    altitude: &Brane<Elevation>,
    continentality: &Brane<f64>,
    temperature_average: &Brane<Temperature>,
    ocean_tiles: &Brane<bool>,
) -> Month {
    let temperature = temperature_at_altitude(
        &temperature_at_ocean_level(solar_time, temperature_average, continentality),
        &altitude,
    );
    let evaporation_potential = evapotranspiration_potential(&temperature);
    Month::new(
        temperature.clone(),
        rainfall(
            &altitude,
            &temperature,
            &evaporation_potential,
            &ocean_tiles,
        ),
        evaporation_potential,
    )
}

/* # cosmos */

pub struct Cosmos {
    pub altitude: Brane<Elevation>,
    pub charts: Brane<Chart>,
}

impl Cosmos {
    /// initialise cosmos with given bedrock brane
    pub fn new(altitude: Brane<Elevation>, charts: Brane<Chart>) -> Self {
        Self { altitude, charts }
    }

    pub fn simulate() -> Self {
        let elevation = Brane::<Elevation>::load("elevation".to_string()).downgrade(4);
        let resolution = elevation.resolution;
        let ocean_lv = ocean_level(&elevation);

        // # establish small branes
        let resolution_small = Resolution::confine(216); // mostly for rainfall
        let elevation_small = elevation.upscale(resolution_small);
        let ocean_tiles_small = ocean_tiles(&elevation_small, ocean_lv);
        let altitude_small = altitude_above_ocean_level(&elevation_small, ocean_lv);
        let temperature_average_small = temperature_average(resolution_small);
        let continentality_small = continentality(
            &altitude_small,
            &temperature_average_small,
            &ocean_tiles_small,
        );

        // # small run
        trace!("simulating atmospheric condidtions");
        let year_small = (0..18)
            .map(|sol| {
                simulate_month(
                    sol as f64 / 18.,
                    &altitude_small,
                    &continentality_small,
                    &temperature_average_small,
                    &ocean_tiles_small,
                )
            })
            .collect::<Vec<Month>>();

        // # upscale
        trace!("upscaling results");
        let altitude = altitude_above_ocean_level(&elevation, ocean_lv);
        let continentality = continentality_small.upscale_raw(resolution);
        let year = year_small
            .into_iter()
            .map(|month| {
                month.upscale(
                    &altitude_small,
                    &continentality_small,
                    &altitude,
                    &continentality,
                )
            })
            .collect::<Vec<Month>>();

        Self::new(
            elevation,
            Brane::create_by_index(resolution, |j| {
                Chart::new(
                    year.iter()
                        .map(|month| month.temp.grid[j])
                        .collect::<Vec<Temperature>>(),
                    year.iter()
                        .map(|month| month.rain.grid[j])
                        .collect::<Vec<Precipitation>>(),
                    year.iter()
                        .map(|month| month.pevt.grid[j])
                        .collect::<Vec<Precipitation>>(),
                )
            }),
        )
    }

    pub fn vege(&self) -> Brane<Option<Vege>> {
        let olv = ocean_level(&self.altitude);
        let ocean_tiles = self.altitude.operate_by_value_ref(|value| value < &olv);
        self.charts.operate_by_index(|j| {
            if ocean_tiles.grid[j] {
                None
            } else {
                Some(Vege::from(&Zone::from(&self.charts.grid[j])))
            }
        })
    }
}
