#[allow(unused_imports)]
use crate::carto::{colour as clr, render::Renderable};
use crate::{
    carto::brane::{Brane, Resolution},
    climate::{
        chart::{Chart, Zone},
        circulation::{
            altitude_above_ocean_level, continentality, evapotranspiration_potential, ocean_tiles,
            rainfall, temperature_at_altitude, temperature_at_ocean_level, temperature_average,
            OCNLV,
        },
        vegetation::{Vege, ZoneType},
    },
    regression::predict_brane,
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

    pub fn render(&self) {
        self.temp.stats();
        self.temp
            .upscale(Resolution::confine(324))
            .render("temp".to_string(), clr::CelciusInk);
        self.rain.stats();
        self.rain
            .upscale(Resolution::confine(324))
            .render("rain".to_string(), clr::MoonMeterInk);
        self.pevt.stats();
        self.pevt
            .upscale(Resolution::confine(324))
            .render("pevt".to_string(), clr::MoonMeterInk);
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
        altitude,
    );
    let evaporation_potential = evapotranspiration_potential(&temperature);
    Month::new(
        temperature.clone(),
        rainfall(
            altitude,
            &temperature,
            &evaporation_potential,
            continentality,
            ocean_tiles,
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

    pub fn load() -> Self {
        let elevation = Brane::<Elevation>::load("elevation".to_string());
        let resolution = elevation.resolution;
        Self::new(
            elevation,
            Brane::create_by_index(resolution, |_| Chart::new(vec![], vec![], vec![])),
        )
    }

    pub fn simulate(self) -> Self {
        let elevation = self.altitude;
        let resolution = elevation.resolution;
        let ocean_lv = Elevation::confine(OCNLV);

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
        // continentality_small.stats_raw();
        // continentality_small.render("cont".to_string(), clr::HueInk::new(0.08, 0.92));

        // # small run
        trace!("simulating atmospheric condidtions");
        let year_len = 24;
        let year_small = (0..year_len).map(|sol| {
            simulate_month(
                sol as f64 / year_len as f64,
                &altitude_small,
                &continentality_small,
                &temperature_average_small,
                &ocean_tiles_small,
            )
        });

        // # upscale
        let altitude = altitude_above_ocean_level(&elevation, ocean_lv);
        let continentality = continentality_small.upscale_raw(resolution);
        let year = year_small
            .map(|month| {
                month.upscale(
                    &altitude_small,
                    &continentality_small,
                    &altitude,
                    &continentality,
                )
            })
            .collect::<Vec<Month>>();

        //year[0].render();

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
        let ocean_tiles = self
            .altitude
            .operate_by_value_ref(|value| value.release() < OCNLV);
        self.charts.operate_by_index(|j| {
            if ocean_tiles.grid[j] {
                None
            } else {
                Some(Vege::from(ZoneType::from(Zone::from(&self.charts.grid[j]))))
            }
        })
    }
}
