use ilmarien::geology as glg;
use ilmarien::imaging::{colour as clr, render::Renderable};
use log::info;
use pretty_env_logger;

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    for seed in 0..1 {
        let elevation = glg::elevation_generate(324 + seed, seed as u32);
        elevation.save();
        elevation.render(clr::ElevationInk);
    }
    info!("simulation completed")
}
