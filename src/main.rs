use ilmarien::cartography::{colour as clr, render::Renderable};
use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};

use log::info;
use petgraph::dot::Dot;
use pretty_env_logger;

#[allow(dead_code)]
fn test_short() {}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 216;
    let elevation = glg::elevation_generate(res, 0);
    // elevation.render(clr::ElevationInk);

    let insolation = rad::insolation_calculate(res);
    let ocean = hdr::ocean_initialise(res, &elevation);
    let surface_type = srf::surface_type_calculate(res, &ocean);
    let surface_level = srf::surface_level_calculate(res, &elevation, &ocean);

    let temperature = rad::temperature_calculate(res / 3, &insolation, &surface_type);
    // temperature.stats();
    let pressure = rad::pressure_calculate(res / 3, &temperature);

    let evaporation = hdr::evaporation_calculate(res, &surface_type, &temperature, &pressure);
    // evaporation.stats();
    // evaporation.render(clr::HueInk::new(0.84, 0.94));
    //pressure.render(clr::PresInk);

    //let pressure_graph = rad::pressure_gradient(&pressure, &surface_level);
    //println!("{:?}", Dot::new(&graph));
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("simulation completed")
}
