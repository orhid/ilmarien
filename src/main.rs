#[allow(unused_imports)]
use ilmarien::{
    carto::{brane::Resolution, colour as clr, render::Renderable},
    climate::geology::ocean_level,
    units::Unit,
};
use log::info;

// use std::thread;

#[allow(dead_code)]
fn gen_terrain(res: Resolution, seed: u32) {
    use ilmarien::climate::geology::bedrock_elevation;

    let altitude = bedrock_elevation(res, seed);
    let variable = format!("{}-alt", seed);
    altitude.stats();
    altitude.save(variable.clone());
    altitude.render(variable, clr::TopographyInk::new(ocean_level(&altitude)));
}

#[allow(dead_code)]
fn run_once(res: Resolution, seed: u32) {
    use ilmarien::climate::simulation::Cosmos;
    let cosmos = Cosmos::simulate(res, seed);

    let altitude = cosmos.altitude.clone();
    altitude.render(
        format!("{}-alt", seed),
        clr::TopographyInk::new(ocean_level(&cosmos.altitude)),
    );
    let vege = cosmos.vege();
    vege.render(format!("{}-vege", seed), clr::KoppenInk);
    info!("finished simulation at {}", seed);
}

/*
#[allow(dead_code)]
fn run_many() {
    let res: usize = 216;

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    let (j, k) = (3, 7);
    let count = 18;
    for s in 0..count {
        let seed = j * s + k;
        // spin up another thread
        children.push(thread::spawn(move || {
            run_once(res, seed);
        }));
    }

    for child in children {
        // wait for the thread to finish
        let _ = child.join();
    }
}
*/

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    //gen_terrain(Resolution::confine(324), 0);
    run_once(Resolution::confine(432), 6);
    info!("computation completed")
}
