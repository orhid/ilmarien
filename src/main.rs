#[allow(unused_imports)]
use ilmarien::{
    carto::{colour as clr, render::Renderable},
    climate::{
        cosmos as csm, geology as glg, hydrology as hdr, radiation as rad, simulation as sim,
    },
};
use log::info;
use pretty_env_logger;
use std::thread;

#[allow(dead_code)]
fn test_short() {
    //
}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 144;

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    let begin = 0;
    let count = 6;
    for seed in begin..begin + count {
        // spin up another thread
        children.push(thread::spawn(move || {
            sim::full_simulation(res, seed);
        }));
    }

    for child in children {
        // wait for the thread to finish. Returns a result
        let _ = child.join();
    }
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("computation completed")
}
