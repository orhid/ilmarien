#[allow(unused_imports)]
use ilmarien::{
    carto::{colour as clr, render::Renderable},
    climate::cosmos::Cosmos,
};
use log::info;
use std::thread;

#[allow(dead_code)]
fn test() {}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 432;

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    let (j, k) = (2u32.pow(8), 3u32.pow(6));
    let count = 1;
    for s in 0..count {
        let seed = j * s + k;
        // spin up another thread
        children.push(thread::spawn(move || {
            let cosmos = Cosmos::sim_new(res, seed);
            let mut vege = cosmos.vege();
            vege.variable = format!("vege-{}", seed);
            vege.render(clr::KoppenInk);
            info!("finished simulation at {}", seed);
        }));
    }

    for child in children {
        // wait for the thread to finish
        let _ = child.join();
    }
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    test_sim();
    info!("computation completed")
}
