use ilmarien::geology as glg;
use ilmarien::imaging::{colour as clr, render::Renderable};
use ilmarien::insolation as isl;
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
    let insolation = isl::insolation_calculate(324);
    insolation.save();
    insolation.render(clr::HueInk::new(0.0, 0.72));
}

#[allow(dead_code)]
fn isl_wtch_cmp() {
    for watch in 0..16 {
        let insolation = isl::insolation_watch_calculate(108, watch);
        //insolation.save();
        insolation.render(clr::HueInk::new(0.12, 0.72));
    }
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    isl_wtch_cmp();
    info!("simulation completed")
}
