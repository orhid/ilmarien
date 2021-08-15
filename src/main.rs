use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
use ilmarien::imaging::{cartography as crt, colour as clr, render::Renderable};
use ilmarien::util::constants::*;
use log::info;
use pretty_env_logger;

#[allow(dead_code)]
fn test_radiation() {
    // temperature should be calculated at a way smaller resolution than elevation
    let res: usize = 216;
    let elevation = glg::elevation_generate(res, 4);
    elevation.render(clr::ElevationInk);

    let insolation = rad::insolation_calculate(res);
    let ocean = hdr::ocean_initialise(res, &elevation);
    let surface = srf::surface_calculate(res, &ocean);
    //let albedo = srf::albedo_calculate(res, &surface);
    //let capacitance = srf::capacitance_calculate(res, &surface);

    let mut temperature = rad::temperature_initialise(res / 3, &insolation);
    rad::temperature_diffuse(&mut temperature, &surface);
    // rad::temperature_absorb(&mut temperature, &insolation, &albedo, &capacitance);
    // rad::temperature_radiate(&mut temperature, &capacitance);
    let tmp_dif = temperature.grid.iter().sum::<f64>() / temperature.grid.len() as f64;
    let tmp_min = temperature
        .grid
        .iter()
        .min_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let tmp_max = temperature
        .grid
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    println!("min: {}", tmp_min);
    println!("max: {}", tmp_max);
    println!("abs: {}", tmp_max - tmp_min);
    println!("men: {}", tmp_dif);
    println!("dif: {}", tmp_dif - MID_TEMP);
    temperature.render(clr::TempInk);
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_radiation();
    info!("simulation completed")
}
