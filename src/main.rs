use ilmarien::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
use ilmarien::imaging::{cartography::load, colour as clr, render::Renderable};
use log::info;
use pretty_env_logger;

#[allow(dead_code)]
fn ele_gen() {
    for seed in 0..1 {
        let elevation = glg::elevation_generate(324 + seed, seed as u32);
        elevation.save();
        elevation.render(clr::ElevationInk);
    }
}

#[allow(dead_code)]
fn isl_cmp() {
    let insolation = rad::insolation_calculate(324);
    insolation.save();
    insolation.render(clr::HueInk::new(0.0, 0.72));
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
    let elevation = load("elevation-3456".to_string());
    let insolation = load("insolation-3456".to_string());
    let water = hdr::water_initialise(res, &elevation);
    let surface = srf::surface_calculate(res, &water);
    surface.save();
    let albedo = srf::albedo_calculate(res, &surface);
    let absorbtion = rad::heat_absorbtion_calculate(res, &insolation, &albedo);
    absorbtion.save();
    //absorbtion.render(clr::HueInk::new(0.0, 0.72));
}

#[allow(dead_code)]
fn diffuse_cmp(res: usize) {
    let surface = load(format!("surface-{}", res));
    let absorbtion = load(format!("heat-absorbed-{}", res));
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
