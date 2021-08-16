use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
// use ilmarien::cartography::{colour as clr, render::Renderable};

use log::info;
use petgraph::dot::Dot;
use pretty_env_logger;

#[allow(dead_code)]
fn test_short() {}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 12;
    let elevation = glg::elevation_generate(res, 1);
    // elevation.render(clr::ElevationInk);

    let insolation = rad::insolation_calculate(res);
    let ocean = hdr::ocean_initialise(res, &elevation);
    let surface_type = srf::surface_type_calculate(res, &ocean);
    let surface_level = srf::surface_level_calculate(res, &elevation, &ocean);

    let temperature = rad::temperature_calculate(res / 3, &insolation, &surface_type);
    let pressure = rad::pressure_calculate(res, &temperature, &surface_level);

    let graph = rad::pressure_gradient(&pressure, &surface_level);
    println!("{:?}", Dot::new(&graph));
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("simulation completed")
}
