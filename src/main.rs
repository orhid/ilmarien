use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
use ilmarien::imaging::{cartography as crt, colour as clr, render::Renderable};
use log::info;
use pretty_env_logger;

#[allow(dead_code)]
fn ele_gen() {
    for seed in 0..1 {
        let elevation = glg::elevation_generate(3456 + seed, seed as u32);
        elevation.save();
        //elevation.render(clr::ElevationInk);
    }
}

#[allow(dead_code)]
fn isl_cmp() {
    let insolation = rad::insolation_calculate(3456);
    insolation.save();
    //insolation.render(clr::HueInk::new(0.0, 0.72));
}

#[allow(dead_code)]
fn isl_wtch_cmp() {
    for watch in 0..16 {
        let insolation = rad::insolation_watch_calculate(108, watch);
        //insolation.save();
        insolation.render(clr::HueInk::new(0.12, 0.72));
    }
}

#[allow(dead_code)]
fn absorb_cmp(res: usize) {
    let elevation = crt::Brane::<f64>::load("elevation".to_string());
    let insolation = crt::Brane::<f64>::load("insolation".to_string());
    let water = hdr::ocean_initialise(res, &elevation);
    let surface = srf::surface_calculate(res, &water);
    surface.save();
    let albedo = srf::albedo_calculate(res, &surface);
    let absorbtion = rad::heat_absorbtion_calculate(res, &insolation, &albedo);
    absorbtion.save();
    //absorbtion.render(clr::HueInk::new(0.0, 0.72));
}

#[allow(dead_code)]
fn diffuse_cmp(res: usize) {
    let surface = crt::Brane::<u8>::load("surface".to_string());
    let absorbtion = crt::Brane::<f64>::load("heat-absorbed".to_string());
    let diffusion = rad::heat_diffusion_calculate_all(res, &absorbtion, &surface);
    diffusion.save();
    //diffusion.render(clr::HueInk::new(0.0, 0.72));
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    absorb_cmp(432);
    diffuse_cmp(432);
    info!("simulation completed")
}
