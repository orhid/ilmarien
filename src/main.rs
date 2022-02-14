#[allow(unused_imports)]
use ilmarien::carto::{colour as clr, render::Renderable};
use ilmarien::climate::{cosmos as csm, geology as glg};
use log::info;
use pretty_env_logger;
use std::thread;

#[allow(dead_code)]
fn test() {}

#[allow(dead_code)]
fn test_sim() {
    let res: usize = 432;

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    let begin = 837;
    let count = 4;
    for seed in begin..begin + count {
        // spin up another thread
        children.push(thread::spawn(move || {
            let mut cosmos = csm::Cosmos::new(&glg::bedrock_level(res, seed));
            cosmos.sim_climate(2, 12);
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
    //test();
    info!("computation completed")
}
