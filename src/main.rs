use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
// use ilmarien::imaging::{cartography as crt, colour as clr, render::Renderable};
use log::info;
use petgraph::dot::Dot;
use pretty_env_logger;

#[allow(dead_code)]
fn test_short() {}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 6;
    let elevation = glg::elevation_generate(res, 0);
    // elevation.render(clr::ElevationInk);

    let insolation = rad::insolation_calculate(res);
    let ocean = hdr::ocean_initialise(res, &elevation);
    let surface = srf::surface_calculate(res, &ocean);

    let temperature = rad::temperature_calculate(res / 3, &insolation, &surface);
    let pressure = rad::pressure_calculate(res / 3, &temperature);

    let graph = rad::pressure_gradient(&pressure, &temperature, &elevation);
    println!("{:?}", Dot::new(&graph));
    // pressure.render(clr::TempInk);
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("simulation completed")
}
