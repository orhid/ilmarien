use ilmarien::{
    carto::{colour as clr, render::Renderable},
    climate::{cosmos as csm, geology as glg, hydrology as hdr, radiation as rad},
};

use log::info;
use pretty_env_logger;

#[allow(dead_code)]
fn test_short() {
    let bedrock = glg::bedrock_level(108, 0);
    bedrock.render(clr::ElevationInk);
}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 216;
    let seed = 0;
    let bedrock = glg::bedrock_level(res, seed);
    let cosmos = csm::initialise(&bedrock);
    let elevation = csm::elevation(&cosmos);
    let surface = csm::surface(&cosmos);

    let insolation = rad::insolation(res / 3, 1.0);
    let temperature = rad::temperature(&insolation, &surface);
    let pressure = rad::pressure(&temperature);

    let evaporation = hdr::evaporation(&pressure, &surface, &temperature);
    let pressure_flux = rad::pressure_flux(&pressure);
    let rainfall = hdr::rainfall(&elevation, &evaporation, &pressure_flux);
    rainfall.render(clr::HueInk::new(0.54, 0.94));
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("simulation completed")
}
