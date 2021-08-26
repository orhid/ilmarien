#[allow(unused_imports)]
use ilmarien::{
    carto::{colour as clr, render::Renderable},
    climate::{
        cosmos as csm, geology as glg, hydrology as hdr, radiation as rad, simulation as sim,
    },
};
use log::info;
use pretty_env_logger;

#[allow(dead_code)]
fn test_short() {
    //
}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 216;
    let seed = 0;

    sim::full_simulation(res, seed);
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("computation completed")
}
