// use ilmarien::cartography::{colour as clr, render::Renderable};
// use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};

use log::info;
use pretty_env_logger;

#[allow(dead_code)]
fn test_short() {}

/*
#[allow(dead_code)]
fn test_sim() {
    let res: usize = 216;
    let seed = 0;
    let elevation = glg::bedrock_level(res, seed);

    let insolation = rad::insolation_calculate(res);
    let ocean = hdr::ocean_initialise(res, &elevation);
    let surface_type = srf::surface_type_calculate(res, &ocean);
    let surface_level = srf::surface_level_calculate(res, &elevation, &ocean);

    let temperature = rad::temperature_calculate(res / 3, &insolation, &surface_type);
    let pressure = rad::pressure_calculate(res / 3, &temperature);

    let evaporation = hdr::evaporation_calculate(res / 3, &surface_type, &temperature, &pressure);

    let rainfall = hdr::rainfall(&pressure, &evaporation, &surface_level);
    rainfall.render(clr::HueInk::new(0.54, 0.94));
}
*/

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_short();
    info!("simulation completed")
}
