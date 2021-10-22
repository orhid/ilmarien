#[allow(unused_imports)]
use crate::{
    carto::{brane::Brane, colour as clr, render::Renderable},
    climate::{cosmos as csm, geology as glg, hydrology as hdr, koppen as kpn, radiation as rad},
    vars::*,
};
use log::info;

#[derive(PartialEq)]
enum Time {
    Cosmic,
    Local,
}

#[allow(unused_variables)]
fn single_loop(cosmos: &mut csm::Cosmos, resolution: usize, sol: f64, scale: Time) {
    // temperature
    let surface = cosmos.surface();
    let temperature: Brane<f64>;
    match scale {
        // cosmic scale should be used when sedimant transfer is active
        Time::Cosmic => {
            temperature = (rad::temperature(
                &rad::insolation(if resolution > 216 { 144 } else { 72 }, SOL_DEV),
                &surface,
            ) + rad::temperature(
                &rad::insolation(if resolution > 216 { 144 } else { 72 }, -SOL_DEV),
                &surface,
            )) * 0.5;
        }
        Time::Local => {
            temperature = rad::temperature(
                &rad::insolation(if resolution > 216 { 144 } else { 72 }, sol),
                &surface,
            );
        }
    }

    /*
    cosmos.solidify_snow();
    let icemelt = cosmos.form_glaciers(&temperature);
    */

    // wind
    let pressure = rad::pressure(&temperature);
    let wind = rad::wind(&pressure);
    // TODO: move light sediment through wind

    // rainfall
    let evaporation = hdr::evaporation(&pressure, &surface, &temperature);
    let elevation = cosmos.elevation();
    let mut rainfall = hdr::rainfall(&elevation, &evaporation, &wind);
    // cosmos.snowfall(&mut rainfall, &temperature);

    // rainfall = rainfall + icemelt;
    if scale == Time::Local {
        cosmos.update_kp(&elevation, &(temperature - 273.0), &(rainfall * 162.0));
    }
    // let landflow = cosmos.landflow();
    // let shed = hdr::shed(&landflow, &rainfall);
    // TODO: move sediment through water flow

    // TODO: solidify sediment
}

#[allow(unused_variables)]
pub fn full_simulation(resolution: usize, seed: u32) {
    // initailise bedrock level
    let bedrock = glg::bedrock_level(resolution, seed);

    // TODO: rock types map

    //  initialise cosmic onion
    let mut cosmos = csm::Cosmos::initialise(&bedrock);
    cosmos.variable = format!("cosmos-{}", seed);
    cosmos.render(clr::TopographyInk::new(INIT_OCEAN_LEVEL));

    /*
    for _ in 0..4 {
        single_loop(&mut cosmos, resolution, 0.0, Time::Cosmic);
    }
    */

    let cycle = 12;
    let years = 3;
    for sol in (0..years * cycle)
        .map(|c| SOL_DEV * (std::f64::consts::TAU * c as f64 / cycle as f64).sin())
    {
        single_loop(&mut cosmos, resolution, sol, Time::Local);
    }

    cosmos.variable = format!("cosmos-{}-koppen", seed);
    cosmos.render(clr::KoppenInk);
    info!("finished simulation with seed {}", seed);
}
